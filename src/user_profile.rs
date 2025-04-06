use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::{self, File};
use std::io;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActivityLevel {
    Sedentary,
    LightlyActive,
    ModeratelyActive,
    VeryActive,
    SuperActive,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TargetCalorieCalcStrategy {
    MifflinStJeor,
    KatchMcArdle,
    HarrisBenedict,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Gender {
    Male,
    Female,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserProfile {
    pub name: String,
    pub height: f64,
    pub weight: f64,
    pub age: u32,
    #[serde(default = "default_gender")]
    pub gender: Gender,
    pub activity_level: ActivityLevel,
    pub target_calorie_calc_strategy: TargetCalorieCalcStrategy,
    pub target_calorie: f64,
}

fn default_gender() -> Gender {
    Gender::Male // Default to Male when gender field is missing
}

impl UserProfile {
    pub fn new(
        name: String,
        height: f64,
        weight: f64,
        age: u32,
        gender: Gender,
        activity_level: ActivityLevel,
        target_calorie_calc_strategy: TargetCalorieCalcStrategy,
    ) -> Self {
        let target_calorie = Self::calculate_target_calorie(
            height,
            weight,
            age,
            &gender,
            &activity_level,
            &target_calorie_calc_strategy,
        );
        
        UserProfile {
            name,
            height,
            weight,
            age,
            gender,
            activity_level,
            target_calorie_calc_strategy,
            target_calorie,
        }
    }
    
    pub fn calculate_target_calorie(
        height: f64,
        weight: f64,
        age: u32,
        gender: &Gender,
        activity_level: &ActivityLevel,
        strategy: &TargetCalorieCalcStrategy,
    ) -> f64 {
        // Calculate BMR based on strategy
        let bmr = match strategy {
            TargetCalorieCalcStrategy::MifflinStJeor => {
                let base = 10.0 * weight + 6.25 * height - 5.0 * age as f64;
                match gender {
                    Gender::Male => base + 5.0,
                    Gender::Female => base - 161.0,
                }
            },
            TargetCalorieCalcStrategy::KatchMcArdle => {
                // Estimate lean body mass (simplified as we don't have body fat %)
                let estimated_body_fat_percentage = match gender {
                    Gender::Male => 15.0,  // Rough average for men
                    Gender::Female => 25.0,  // Rough average for women
                };
                
                let lean_body_mass = weight * (100.0 - estimated_body_fat_percentage) / 100.0;
                370.0 + (21.6 * lean_body_mass)
            },
            TargetCalorieCalcStrategy::HarrisBenedict => {
                match gender {
                    Gender::Male => {
                        66.0 + (13.75 * weight) + (5.0 * height) - (6.76 * age as f64)
                    },
                    Gender::Female => {
                        655.0 + (9.56 * weight) + (1.85 * height) - (4.68 * age as f64)
                    },
                }
            },
        };
        
        // Apply activity level multiplier
        let activity_multiplier = match activity_level {
            ActivityLevel::Sedentary => 1.2,
            ActivityLevel::LightlyActive => 1.375,
            ActivityLevel::ModeratelyActive => 1.55,
            ActivityLevel::VeryActive => 1.725,
            ActivityLevel::SuperActive => 1.9,
        };
        
        (bmr * activity_multiplier).round() // Rounded to the nearest whole calorie
    }
}

pub fn load_users() -> Vec<UserProfile> {
    let file_path = "users.yaml";
    if Path::new(file_path).exists() {
        let data = fs::read_to_string(file_path).expect("Unable to read file");
        serde_yaml::from_str(&data).expect("Unable to parse YAML")
    } else {
        Vec::new()
    }
}

pub fn save_users(users: &Vec<UserProfile>) {
    let file_path = "users.yaml";
    let data = serde_yaml::to_string(users).expect("Unable to serialize users");
    let mut file = File::create(file_path).expect("Unable to create file");
    file.write_all(data.as_bytes()).expect("Unable to write data");
}

pub fn select_user(users: &Vec<UserProfile>) -> Option<usize> {
    println!("Select a user:");
    for (i, user) in users.iter().enumerate() {
        println!("{}: {}", i + 1, user.name);
    }
    println!("Enter the number of the user (or 0 to cancel):");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    if let Ok(choice) = input.trim().parse::<usize>() {
        if choice > 0 && choice <= users.len() {
            return Some(choice - 1);
        }
    }
    None
}

pub fn create_user() -> UserProfile {
    println!("Creating a new user:");
    let mut input = String::new();

    println!("Enter name:");
    io::stdin().read_line(&mut input).expect("Failed to read input");
    let name = input.trim().to_string();

    println!("Enter height (in cm):");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    let height: f64 = input.trim().parse().expect("Invalid height");

    println!("Enter weight (in kg):");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    let weight: f64 = input.trim().parse().expect("Invalid weight");

    println!("Enter age:");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    let age: u32 = input.trim().parse().expect("Invalid age");
    
    println!("Select gender:");
    println!("1: Male");
    println!("2: Female");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    let gender = match input.trim() {
        "1" => Gender::Male,
        "2" => Gender::Female,
        _ => panic!("Invalid choice"),
    };

    println!("Select activity level:");
    println!("1: Sedentary");
    println!("2: Lightly Active");
    println!("3: Moderately Active");
    println!("4: Very Active");
    println!("5: Super Active");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    let activity_level = match input.trim() {
        "1" => ActivityLevel::Sedentary,
        "2" => ActivityLevel::LightlyActive,
        "3" => ActivityLevel::ModeratelyActive,
        "4" => ActivityLevel::VeryActive,
        "5" => ActivityLevel::SuperActive,
        _ => panic!("Invalid choice"),
    };

    println!("Select calorie calculation strategy:");
    println!("1: Mifflin-St Jeor");
    println!("2: Katch-McArdle");
    println!("3: Harris-Benedict");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    let target_calorie_calc_strategy = match input.trim() {
        "1" => TargetCalorieCalcStrategy::MifflinStJeor,
        "2" => TargetCalorieCalcStrategy::KatchMcArdle,
        "3" => TargetCalorieCalcStrategy::HarrisBenedict,
        _ => panic!("Invalid choice"),
    };

    UserProfile::new(
        name,
        height,
        weight,
        age,
        gender,
        activity_level,
        target_calorie_calc_strategy,
    )
}

pub fn modify_user(user: &mut UserProfile) {
    println!("Modifying user: {}", user.name);
    let mut input = String::new();

    println!("Enter new name (or press Enter to keep current):");
    io::stdin().read_line(&mut input).expect("Failed to read input");
    if !input.trim().is_empty() {
        user.name = input.trim().to_string();
    }

    println!("Enter new height (or press Enter to keep current):");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    if let Ok(height) = input.trim().parse::<f64>() {
        user.height = height;
    }

    println!("Enter new weight (or press Enter to keep current):");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    if let Ok(weight) = input.trim().parse::<f64>() {
        user.weight = weight;
    }

    println!("Enter new age (or press Enter to keep current):");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    if let Ok(age) = input.trim().parse::<u32>() {
        user.age = age;
    }

    println!("Select new activity level (or press Enter to keep current):");
    println!("1: Sedentary");
    println!("2: Lightly Active");
    println!("3: Moderately Active");
    println!("4: Very Active");
    println!("5: Super Active");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    if let Ok(choice) = input.trim().parse::<u32>() {
        user.activity_level = match choice {
            1 => ActivityLevel::Sedentary,
            2 => ActivityLevel::LightlyActive,
            3 => ActivityLevel::ModeratelyActive,
            4 => ActivityLevel::VeryActive,
            5 => ActivityLevel::SuperActive,
            _ => user.activity_level.clone(),
        };
    }

    println!("Select new gender (or press Enter to keep current):");
    println!("1: Male");
    println!("2: Female");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    if let Ok(choice) = input.trim().parse::<u32>() {
        user.gender = match choice {
            1 => Gender::Male,
            2 => Gender::Female,
            _ => user.gender.clone(),
        };
    }

    println!("Select new calorie calculation strategy (or press Enter to keep current):");
    println!("1: Mifflin-St Jeor");
    println!("2: Katch-McArdle");
    println!("3: Harris-Benedict");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    if let Ok(choice) = input.trim().parse::<u32>() {
        user.target_calorie_calc_strategy = match choice {
            1 => TargetCalorieCalcStrategy::MifflinStJeor,
            2 => TargetCalorieCalcStrategy::KatchMcArdle,
            3 => TargetCalorieCalcStrategy::HarrisBenedict,
            _ => user.target_calorie_calc_strategy.clone(),
        };
    }

    // Recalculate target calories based on the updated user information
    user.target_calorie = UserProfile::calculate_target_calorie(
        user.height,
        user.weight,
        user.age,
        &user.gender,
        &user.activity_level,
        &user.target_calorie_calc_strategy,
    );
    
    println!("Calculated daily target calories: {:.0}", user.target_calorie);
}