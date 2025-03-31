mod user_profile;
mod food_database;
mod daily_log;

use user_profile::UserProfile;
use food_database::FoodDatabase;
use daily_log::DailyLog;

fn main() {
    let mut user_profile = UserProfile::new("male", 180.0, 75.0, 25, "active");
    let mut food_db = FoodDatabase::new();
    let mut daily_log = DailyLog::new();

    println!("Welcome to Yada App!");
    user_profile.update_profile(Some(185.0), None, None, None);
    println!("Updated user profile: {:?}", user_profile);
}

fn FoodDatabaseMenu() {
    println!("Food Database Menu:");
    println!("1. Add a Basic Food Item");
    println!("2. Add a Compsite Food Item");
    println!("3. Exit");
    println!();

    loop {
        let mut choice = String::new();
        println!("Enter your choice: ");
        std::io::stdin().read_line(&mut choice).unwrap();
        match choice.trim() {
            "1" => {
                let mut name = String::new();
                let mut calories = String::new();
                println!("Enter the name of the food item: ");
                std::io::stdin().read_line(&mut name).unwrap();
                println!("Enter the calories per serving: ");
                std::io::stdin().read_line(&mut calories).unwrap();
                let calories: f64 = match calories.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("Invalid input for calories, please try again.");
                        continue;
                    }
                };
                
                food_db.add_basic_food(name.trim(), calories);
            }
            "2" => {
                let mut name = String::new();
                let mut components = Vec::new();
                println!("Enter the name of the composite food item: ");
                std::io::stdin().read_line(&mut name).unwrap();
                loop {
                    let mut component_name = String::new();
                    let mut quantity = String::new();
                    println!("Enter the name of the basic or composite component (or 'done' to finish): ");
                    std::io::stdin().read_line(&mut component_name).unwrap();
                    if component_name.trim() == "done" {
                        break;
                    }
                    
                    results = search_foods(&food_db, component_name.trim());
                    if results.is_empty() {
                        println!("No food item found with that name, please try again.");
                        continue;
                    }
                    println!("Found food items:");
                    let mut i = 1;
                    for (name, calories) in results {
                        println!("{}: {}", i, name);
                        i += 1;
                    }
                    println!("Enter the index of the food item you want to use: ");
                    let mut index = String::new();
                    std::io::stdin().read_line(&mut index).unwrap();
                    let index: usize = match index.trim().parse() {
                        Ok(num) => num,
                        Err(_) => {
                            println!("Invalid input for index, please try again.");
                            continue;
                        }
                    };
                    if index == 0 || index > results.len() {
                        println!("Invalid index, please try again.");
                        continue;
                    }
                    let selected_food = &results[index - 1];

                    println!("Enter the quantity: ");
                    std::io::stdin().read_line(&mut quantity).unwrap();
                    let quantity: f64 = match quantity.trim().parse() {
                        Ok(num) => num,
                        Err(_) => {
                            println!("Invalid input for quantity, please try again.");
                            continue;
                        }
                    };
                    
                    components.push((selected_food.0.clone(), quantity));
                }
                food_db.add_composite_food(name.trim(), components);
            }
            "3" => break,
            _ => println!("Invalid choice, please try again."),
        }
    }
}