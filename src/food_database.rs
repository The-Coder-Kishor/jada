use std::fs;
use std::io;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct FoodDatabase {
    pub basic_foods: Vec<BasicFood>,
    pub composite_foods: Vec<CompositeFood>,
    basic_foods_path: String,
    composite_foods_path: String,
}

impl FoodDatabase {
    pub fn new() -> Self {
        Self {
            basic_foods: Vec::new(),
            composite_foods: Vec::new(),
            basic_foods_path: "data/basic_foods.yaml".to_string(),
            composite_foods_path: "data/composite_foods.yaml".to_string(),
        }
    }

    pub fn load(&mut self) -> Result<(), io::Error> {
        // Load basic foods
        if Path::new(&self.basic_foods_path).exists() {
            let contents = fs::read_to_string(&self.basic_foods_path)?;
            let db: BasicFoodsWrapper = serde_yaml::from_str(&contents)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            self.basic_foods = db.basic_foods;
        }

        // Then load composite foods
        if Path::new(&self.composite_foods_path).exists() {
            let contents = fs::read_to_string(&self.composite_foods_path)?;
            let db: SerializedCompositeFoodsWrapper = serde_yaml::from_str(&contents)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            
            self.composite_foods = Vec::new();
            
            // Process each composite food reference and resolve its components
            for serialized_food in db.composite_foods {
                let mut components = Vec::new();
                
                for component in &serialized_food.components {
                    // Find the basic food with the matching identifier
                    if let Some(basic_food) = self.basic_foods.iter()
                        .find(|bf| bf.identifier == component.food_id) {
                        components.push((basic_food.clone(), component.quantity));
                    } else {
                        eprintln!("Warning: Basic food '{}' referenced in composite food '{}' not found",
                            component.food_id, serialized_food.identifier);
                    }
                }
                
                self.composite_foods.push(CompositeFood {
                    identifier: serialized_food.identifier,
                    keywords: serialized_food.keywords,
                    components,
                });
            }
        }

        Ok(())
    }

    pub fn save(&self) -> Result<(), io::Error> {
        // Ensure directories exist
        if let Some(parent) = Path::new(&self.basic_foods_path).parent() {
            fs::create_dir_all(parent)?;
        }

        // Save basic foods
        let basic_db = BasicFoodsWrapper {
            basic_foods: self.basic_foods.clone(),
        };
        let yaml = serde_yaml::to_string(&basic_db)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&self.basic_foods_path, yaml)?;

        // Convert composite foods to serializable format and save
        let serialized_foods: Vec<SerializedCompositeFood> = self.composite_foods.iter()
            .map(|food| food.to_serialized())
            .collect();
        
        let composite_db = SerializedCompositeFoodsWrapper {
            composite_foods: serialized_foods,
        };
        
        let yaml = serde_yaml::to_string(&composite_db)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&self.composite_foods_path, yaml)?;

        Ok(())
    }

    pub fn search_foods(&self, prefix: &str) -> Vec<(&str, f64)> {
        let prefix = prefix.to_lowercase();
        let mut results = Vec::new();
        
        // Search basic foods
        for food in &self.basic_foods {
            if food.identifier.to_lowercase().starts_with(&prefix) || 
               food.keywords.iter().any(|k| k.to_lowercase().starts_with(&prefix)) {
                results.push((food.identifier.as_str(), food.calories_per_serving));
            }
        }
        
        // Search composite foods
        for food in &self.composite_foods {
            if food.identifier.to_lowercase().starts_with(&prefix) || 
               food.keywords.iter().any(|k| k.to_lowercase().starts_with(&prefix)) {
                results.push((food.identifier.as_str(), food.get_calories()));
            }
        }
        
        results
    }

    pub fn add_basic_food(&mut self, identifier: &str, keywords: Vec<String>, calories_per_serving: f64) -> Result<(), io::Error> {
        // Check if a food with this identifier already exists
        if self.basic_foods.iter().any(|food| food.identifier == identifier) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists, 
                format!("Basic food '{}' already exists", identifier)
            ));
        }
        
        // Create new basic food
        let basic_food = BasicFood {
            identifier: identifier.to_string(),
            keywords,
            calories_per_serving,
        };
        
        // Add to vector
        self.basic_foods.push(basic_food);
        
        // Save to file
        self.save()?;
        
        Ok(())
    }

    pub fn add_composite_food(&mut self, identifier: &str, keywords: Vec<String>, component_ids: Vec<(String, f64)>) -> Result<(), io::Error> {
        // Check if a food with this identifier already exists
        if self.composite_foods.iter().any(|food| food.identifier == identifier) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists, 
                format!("Composite food '{}' already exists", identifier)
            ));
        }
        
        // Build components from IDs
        let mut components = Vec::new();
        
        for (food_id, quantity) in component_ids {
            // Find the basic food with the matching identifier
            if let Some(basic_food) = self.basic_foods.iter()
                .find(|bf| bf.identifier == food_id) {
                components.push((basic_food.clone(), quantity));
            } 
            // Check if it's a composite food
            else if let Some(composite_food) = self.composite_foods.iter()
                .find(|cf| cf.identifier == food_id) {
                // Add all basic components from the composite food with adjusted quantities
                for (basic_food, comp_quantity) in &composite_food.components {
                    let adjusted_quantity = comp_quantity * quantity;
                    components.push((basic_food.clone(), adjusted_quantity));
                }
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound, 
                    format!("Food '{}' not found", food_id)
                ));
            }
        }
        
        // Create new composite food
        let composite_food = CompositeFood {
            identifier: identifier.to_string(),
            keywords,
            components,
        };
        
        // Add to vector
        self.composite_foods.push(composite_food);
        
        // Save to file
        self.save()?;
        
        Ok(())
    }
    
    // Helper method to get a basic food by identifier
    pub fn get_basic_food(&self, identifier: &str) -> Option<&BasicFood> {
        self.basic_foods.iter().find(|f| f.identifier == identifier)
    }
    
    // Helper method to get a composite food by identifier
    pub fn get_composite_food(&self, identifier: &str) -> Option<&CompositeFood> {
        self.composite_foods.iter().find(|f| f.identifier == identifier)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicFood {
    pub identifier: String,
    pub keywords: Vec<String>,
    pub calories_per_serving: f64,
}

impl BasicFood {
    pub fn get_calories(&self) -> f64 {
        self.calories_per_serving
    }
}

#[derive(Debug, Clone)]
pub struct CompositeFood {
    pub identifier: String,
    pub keywords: Vec<String>,
    pub components: Vec<(BasicFood, f64)>, // (BasicFood, quantity)
}

impl CompositeFood {
    pub fn get_calories(&self) -> f64 {
        self.components
            .iter()
            .map(|(food, qty)| food.get_calories() * qty)
            .sum()
    }
    
    // Convert to a serializable format
    fn to_serialized(&self) -> SerializedCompositeFood {
        SerializedCompositeFood {
            identifier: self.identifier.clone(),
            keywords: self.keywords.clone(),
            components: self.components.iter().map(|(basic, qty)| {
                FoodComponent {
                    food_id: basic.identifier.clone(),
                    quantity: *qty,
                }
            }).collect(),
        }
    }
}

// Simple wrapper structs for YAML file format

#[derive(Serialize, Deserialize)]
struct BasicFoodsWrapper {
    basic_foods: Vec<BasicFood>,
}

#[derive(Serialize, Deserialize)]
struct FoodComponent {
    food_id: String,
    quantity: f64,
}

#[derive(Serialize, Deserialize)]
struct SerializedCompositeFood {
    identifier: String,
    keywords: Vec<String>,
    components: Vec<FoodComponent>,
}

#[derive(Serialize, Deserialize)]
struct SerializedCompositeFoodsWrapper {
    composite_foods: Vec<SerializedCompositeFood>,
}