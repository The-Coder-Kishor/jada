use std::fs;
use std::io;
use std::io::Write; // Add this import for flush() method
use std::path::Path;
use serde::{Serialize, Deserialize};
use reqwest;
use scraper::{Html, Selector};
use serde_json::json;

#[derive(Debug)]
pub struct FoodDatabase {
    pub basic_foods: Vec<BasicFood>,
    pub composite_foods: Vec<CompositeFood>,
    basic_foods_path: String,
    composite_foods_path: String,
    ollama_endpoint: String,
}

impl FoodDatabase {
    pub fn new() -> Self {
        Self {
            basic_foods: Vec::new(),
            composite_foods: Vec::new(),
            basic_foods_path: "data/basic_foods.yaml".to_string(),
            composite_foods_path: "data/composite_foods.yaml".to_string(),
            ollama_endpoint: "http://localhost:11434/api/generate".to_string(),
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

    // Enhanced website scraping method
    pub async fn scrape_website(&self, url: &str) -> Result<String, reqwest::Error> {
        println!("Sending request to URL: {}", url);
        
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .build()?;
        
        println!("Sending HTTP request...");
        let response = client.get(url).send().await?;
        println!("Received response: {} {}", response.status().as_u16(), response.status().as_str());
        
        println!("Reading response body...");
        let html_content = response.text().await?;
        println!("Raw HTML length: {} characters", html_content.len());
        
        println!("Parsing HTML document...");
        let document = Html::parse_document(&html_content);
        
        let mut text_content = String::new();
        
        println!("Extracting page content...");
        
        // Extract page title
        if let Some(title_elem) = document.select(&Selector::parse("title").unwrap_or_else(|_| Selector::parse("head").unwrap())).next() {
            text_content.push_str(&format!("Page Title: {}\n\n", title_elem.text().collect::<Vec<_>>().join(" ")));
        }
        
        // Extract meta description if available
        if let Some(meta_desc) = document.select(&Selector::parse("meta[name='description']").unwrap_or_else(|_| Selector::parse("meta").unwrap())).next() {
            if let Some(content) = meta_desc.value().attr("content") {
                text_content.push_str(&format!("Page Description: {}\n\n", content));
            }
        }

        let p_selector = Selector::parse("p").unwrap();
        let p_count = document.select(&p_selector).count();
        println!("Found {} paragraph elements", p_count);
        
        for paragraph in document.select(&p_selector) {
            text_content.push_str(&paragraph.text().collect::<Vec<_>>().join(" "));
            text_content.push_str("\n\n");
        }
        
        for i in 1..7 {
            let h_selector = Selector::parse(&format!("h{}", i)).unwrap();
            for heading in document.select(&h_selector) {
                text_content.push_str(&format!("Heading: {}\n", heading.text().collect::<Vec<_>>().join(" ")));
            }
        }
        
        // Extract lists
        let li_selector = Selector::parse("li").unwrap();
        let li_count = document.select(&li_selector).count();
        println!("Found {} list items", li_count);
        
        for list_item in document.select(&li_selector) {
            text_content.push_str("• ");
            text_content.push_str(&list_item.text().collect::<Vec<_>>().join(" "));
            text_content.push_str("\n");
        }
        
        // Extract divs if not enough content
        if text_content.len() < 100 {
            println!("Not enough content found, attempting to extract from divs...");
            let div_selector = Selector::parse("div").unwrap();
            
            for div in document.select(&div_selector) {
                // Skip empty divs or those with only whitespace
                let div_text = div.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !div_text.is_empty() {
                    text_content.push_str(&div_text);
                    text_content.push_str("\n\n");
                }
            }
        }
        
        // Extract from <main> tag if available
        if let Ok(main_selector) = Selector::parse("main") {
            if let Some(main_elem) = document.select(&main_selector).next() {
                text_content.push_str("Main Content:\n");
                text_content.push_str(&main_elem.text().collect::<Vec<_>>().join(" "));
                text_content.push_str("\n\n");
            }
        }
        
        // Extract from <article> tag if available
        if let Ok(article_selector) = Selector::parse("article") {
            for article in document.select(&article_selector) {
                text_content.push_str("Article Content:\n");
                text_content.push_str(&article.text().collect::<Vec<_>>().join(" "));
                text_content.push_str("\n\n");
            }
        }
        
        // Check if we have sufficient content
        println!("Extracted text content length: {} characters", text_content.len());
        if text_content.is_empty() {
            println!("WARNING: No content extracted. Returning raw HTML text.");
            // Extract text directly from the body as fallback
            if let Some(body) = document.select(&Selector::parse("body").unwrap()).next() {
                return Ok(body.text().collect::<Vec<_>>().join(" "));
            } else {
                return Ok("Failed to extract meaningful content from webpage.".to_string());
            }
        }
        
        Ok(text_content)
    }
    /// Generates basic food data from a website URL
    async fn generate_basic_food_from_website(&self, url: &str) -> Result<BasicFood, io::Error> {
        // First, scrape the website content
        let website_content = match self.scrape_website(url).await {
            Ok(content) => {
                // Add debug output to print the scraped content length
                println!("Successfully scraped website. Content length: {} characters", content.len());
                
                // Print a preview of the content to help with debugging
                if !content.is_empty() {
                    let preview_length = std::cmp::min(200, content.len());
                    println!("Content preview: \n{}", &content[..preview_length]);
                    
                    if content.len() > 200 {
                        println!("... (content truncated, total length: {})", content.len());
                    }
                } else {
                    println!("Warning: Scraped content is empty");
                }
                
                content
            },
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to scrape website: {}", e))),
        };
        
        println!("Generating food data using Ollama LLM...");
        
        // Then use the LLM to generate food data
        let food_data = match self.generate_food_data_with_ollama(&website_content).await {
            Ok(data) => data,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to generate food data: {}", e))),
        };
        
        Ok(food_data)
    }

    /// Uses Ollama with Llama 3.1 to generate food data from website content
    async fn generate_food_data_with_ollama(&self, website_content: &str) -> Result<BasicFood, reqwest::Error> {
        // Create a more robust prompt that handles various webpage formats
        let prompt = format!(
            "You are a nutrition expert analyzing website content to extract food information.

            Your task is to extract or infer the following about a food item from the provided text:
            1. NAME: What food is being described? (Use a clear, concise identifier)
            2. KEYWORDS: What category/type of food is it? (e.g., fruit, protein, dessert, etc.)
            3. CALORIES: How many calories per serving? (Make a reasonable estimate if not stated)

            Even if the information isn't explicitly stated, use your knowledge to make educated guesses.
            If the page discusses multiple foods, focus on the main food item.

            WEBPAGE CONTENT:
            {}

            RESPOND ONLY WITH:
            identifier: [food name in snake_case]
            keywords: [3-5 relevant keywords]
            calories_per_serving: [number]

            No other text or explanations needed.",
            // Allow more content to be processed by splitting into chunks if necessary
            if website_content.len() > 8000 {
                let mut content = String::new();
                let chunks = website_content.as_bytes().chunks(7500);
                for (i, chunk) in chunks.enumerate().take(2) { // Take just 2 chunks (beginning and middle)
                    if i == 0 {
                        content.push_str(&String::from_utf8_lossy(chunk));
                    } else if i == 1 {
                        content.push_str("\n...[content truncated]...\n");
                        // Add final part of the content
                        if let Some(last_part) = website_content.as_bytes().chunks(7500).last() {
                            content.push_str(&String::from_utf8_lossy(last_part));
                        }
                        break;
                    }
                }
                content
            } else {
                website_content.to_string()
            }
        );

        // Create the request payload for Ollama API
        let payload = json!({
            "model": "llama3.1",
            "prompt": prompt,
            "stream": false,
            "temperature": 0.1,
            "max_tokens": 8192
        });

        println!("Sending request to Ollama LLM...");
        
        // Make the API request to Ollama
        let client = reqwest::Client::new();
        let response = client.post(&self.ollama_endpoint)
            .json(&payload)
            .send()
            .await?;

        let response_data: serde_json::Value = response.json().await?;
        let llm_response = response_data["response"].as_str()
            .unwrap_or("Failed to parse response");
        
        println!("Received response from LLM. Processing...");

        // Enhanced parsing with better error handling and fallbacks
        let mut identifier = String::new();
        let mut keywords = Vec::new();
        let mut calories = 0.0;

        // Parse the response line by line
        for line in llm_response.lines() {
            let line = line.trim();
            
            // Extract identifier
            if line.to_lowercase().starts_with("identifier:") {
                identifier = line.splitn(2, ':').nth(1)
                    .unwrap_or("").trim()
                    .replace(" ", "_")
                    .to_lowercase();
            }
            
            // Extract keywords with better handling
            if line.to_lowercase().starts_with("keywords:") {
                let kw_part = line.splitn(2, ':').nth(1).unwrap_or("").trim();
                // Handle both comma-separated and bracket formats
                let clean_kw = kw_part
                    .trim_start_matches('[')
                    .trim_end_matches(']');
                    
                keywords = clean_kw.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            
            // Extract calories with better number parsing
            if line.to_lowercase().starts_with("calories_per_serving:") {
                // Try different number formatting options
                let num_part = line.splitn(2, ':').nth(1).unwrap_or("").trim();
                
                // First try a direct parse
                if let Ok(val) = num_part.parse::<f64>() {
                    calories = val;
                } else {
                    // Try extracting just the first number in the string
                    let num_regex = regex::Regex::new(r"(\d+(?:\.\d+)?)")
                        .unwrap_or_else(|_| regex::Regex::new(r"\d+").unwrap());
                    
                    if let Some(caps) = num_regex.captures(num_part) {
                        if let Some(m) = caps.get(1) {
                            if let Ok(val) = m.as_str().parse::<f64>() {
                                calories = val;
                            }
                        }
                    }
                }
            }
        }
        
        // Apply fallbacks if data is missing
        
        // 1. Handle missing identifier
        if identifier.is_empty() {
            // Try to extract a food name from the first 1000 characters
            let preview = if website_content.len() > 1000 {
                &website_content[0..1000]
            } else {
                website_content
            };
            
            // Look for food-related keywords in content
            let food_indicators = ["food", "recipe", "dish", "meal", "nutrition", "calories", "serving"];
            for line in preview.lines() {
                if food_indicators.iter().any(|&word| line.to_lowercase().contains(word)) {
                    identifier = line.trim()
                        .chars()
                        .take(30)
                        .collect::<String>()
                        .trim()
                        .to_lowercase()
                        .replace(" ", "_");
                    break;
                }
            }
            
            // If still empty, use generic name
            if identifier.is_empty() {
                identifier = "food_item".to_string();
            }
        }
        
        // 2. Handle missing keywords
        if keywords.is_empty() {
            // Extract most frequent non-common words from content
            let common_words = ["the", "and", "a", "an", "in", "on", "at", "of", "to", "for", "with", "this", "that"];
            let mut word_counts = std::collections::HashMap::new();
            
            for word in website_content.split_whitespace()
                .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
                .filter(|w| w.len() > 3 && !common_words.contains(&w.as_str()))
            {
                *word_counts.entry(word).or_insert(0) += 1;
            }
            
            // Sort by count and take top 5
            let mut word_vec: Vec<_> = word_counts.into_iter().collect();
            word_vec.sort_by(|a, b| b.1.cmp(&a.1));
            
            keywords = word_vec.into_iter()
                .take(5)
                .map(|(word, _)| word)
                .collect();
        }
        
        // 3. Handle missing calories
        if calories == 0.0 {
            // Try to find any number between 50-800 (reasonable calorie range)
            let num_regex = regex::Regex::new(r"(\d+(?:\.\d+)?)")
                .unwrap_or_else(|_| regex::Regex::new(r"\d+").unwrap());
            
            for cap in num_regex.captures_iter(website_content) {
                if let Some(m) = cap.get(1) {
                    if let Ok(val) = m.as_str().parse::<f64>() {
                        if val >= 50.0 && val <= 800.0 {
                            calories = val;
                            break;
                        }
                    }
                }
            }
            
            // If still no calories, use default
            if calories == 0.0 {
                calories = 100.0;
            }
        }
        
        // Ensure identifier format is valid
        identifier = identifier
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();
        
        if identifier.is_empty() {
            identifier = "food_item".to_string();
        }
        
        println!("Extracted food data:");
        println!("  Identifier: {}", identifier);
        println!("  Keywords: {:?}", keywords);
        println!("  Calories per serving: {}", calories);
        
        // Create and return the BasicFood struct
        Ok(BasicFood {
            identifier,
            keywords,
            calories_per_serving: calories,
        })
    }
    
    /// Modify the food data before adding (editor mode)
    pub async fn add_food_from_website_with_edit(&mut self, url: &str) -> Result<Option<BasicFood>, io::Error> {
        println!("Scraping food information from {}...", url);
        
        let mut food_data = self.generate_basic_food_from_website(url).await?;
        
        if self.basic_foods.iter().any(|food| food.identifier == food_data.identifier) {
            println!("Warning: A food with identifier '{}' already exists", food_data.identifier);
            print!("Would you like to use a different identifier? (y/n): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if input.trim().to_lowercase() == "y" {
                print!("Enter new identifier: ");
                io::stdout().flush()?;
                
                let mut new_id = String::new();
                io::stdin().read_line(&mut new_id)?;
                food_data.identifier = new_id.trim().to_string();
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists, 
                    format!("Basic food '{}' already exists", food_data.identifier)
                ));
            }
        }
        
        // Display the generated food data and ask for confirmation
        println!("\nGenerated food information:");
        println!("  1. Identifier: {}", food_data.identifier);
        println!("  2. Keywords: [{}]", food_data.keywords.join(", "));
        println!("  3. Calories per serving: {}", food_data.calories_per_serving);
        
        // Ask if the user wants to edit the data
        print!("\nWould you like to edit this information? (y/n): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "y" {
            // Edit mode
            loop {
                println!("\nCurrent food information:");
                println!("  1. Identifier: {}", food_data.identifier);
                println!("  2. Keywords: [{}]", food_data.keywords.join(", "));
                println!("  3. Calories per serving: {}", food_data.calories_per_serving);
                println!("  4. Done editing");
                
                print!("\nSelect an option to edit (1-4): ");
                io::stdout().flush()?;
                
                let mut choice = String::new();
                io::stdin().read_line(&mut choice)?;
                
                match choice.trim() {
                    "1" => {
                        print!("Enter new identifier: ");
                        io::stdout().flush()?;
                        
                        let mut new_id = String::new();
                        io::stdin().read_line(&mut new_id)?;
                        food_data.identifier = new_id.trim().to_string();
                    },
                    "2" => {
                        print!("Enter new keywords (comma-separated): ");
                        io::stdout().flush()?;
                        
                        let mut new_keywords = String::new();
                        io::stdin().read_line(&mut new_keywords)?;
                        
                        food_data.keywords = new_keywords.trim()
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    },
                    "3" => {
                        print!("Enter new calories per serving: ");
                        io::stdout().flush()?;
                        
                        let mut new_calories = String::new();
                        io::stdin().read_line(&mut new_calories)?;
                        
                        if let Ok(cal) = new_calories.trim().parse::<f64>() {
                            food_data.calories_per_serving = cal;
                        } else {
                            println!("Invalid number. Calories not updated.");
                        }
                    },
                    "4" => break,
                    _ => println!("Invalid option. Please try again."),
                }
            }
        }
        
        // Ask for final confirmation
        print!("\nWould you like to add this food to the database? (y/n): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "y" {
            // Add to vector
            let food_clone = food_data.clone();
            self.basic_foods.push(food_data);
            
            // Save to file
            self.save()?;
            
            println!("Food added successfully!");
            Ok(Some(food_clone))
        } else {
            println!("Food not added.");
            Ok(None)
        }
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