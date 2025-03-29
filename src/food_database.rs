#[derive(Debug)]
pub struct FoodDatabase {
    pub basic_foods: Vec<BasicFood>,
    pub composite_foods: Vec<CompositeFood>,
}

impl FoodDatabase {
    pub fn new() -> Self {
        Self {
            basic_foods: Vec::new(),
            composite_foods: Vec::new(),
        }
    }

    pub fn load(&mut self) {
        // Load food data
    }

    pub fn save(&self) {
        // Save food data
    }
}

#[derive(Debug)]
pub struct BasicFood {
    pub name: String,
    pub calories_per_serving: f64,
}

impl BasicFood {
    pub fn get_calories(&self) -> f64 {
        self.calories_per_serving
    }
}

#[derive(Debug)]
pub struct CompositeFood {
    pub name: String,
    pub components: Vec<(BasicFood, f64)>, // (BasicFood, quantity)
}

impl CompositeFood {
    pub fn get_calories(&self) -> f64 {
        self.components
            .iter()
            .map(|(food, qty)| food.get_calories() * qty)
            .sum()
    }
}
