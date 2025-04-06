mod user_profile;
mod food_database;
mod food_log;

use user_profile::{create_user, load_users, modify_user, save_users, select_user, UserProfile};
use food_database::FoodDatabase;
use food_log::{FoodLog, get_calorie_summary};
use std::io;

fn main() {
    let mut users = load_users();

    loop {
        println!("\nUser Management System");
        println!("1. List Users");
        println!("2. Add New User");
        println!("3. Modify Existing User");
        println!("4. User Session");
        println!("5. Save and Exit");
        println!("Select an option (1-5): ");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        match input.trim() {
            "1" => {
                list_users(&users);
            }
            "2" => {
                let new_user = create_user();
                users.push(new_user);
                println!("User added successfully.");
            }
            "3" => {
                if let Some(index) = select_user(&users) {
                    modify_user(&mut users[index]);
                    println!("User modified successfully.");
                } else {
                    println!("No user selected.");
                }
            }
            "4" => {
                user_session(&mut users);
            }
            "5" => {
                save_users(&users);
                println!("Users saved. Exiting...");
                break;
            }
            _ => println!("Invalid option. Please try again."),
        }
    }
}

fn list_users(users: &[UserProfile]) {
    if users.is_empty() {
        println!("No users found.");
    } else {
        println!("Users:");
        for (i, user) in users.iter().enumerate() {
            println!("{}: {}", i + 1, user.name);
        }
    }
}

fn user_session(users: &mut Vec<UserProfile>) {
    if users.is_empty() {
        println!("No users available. Please add a user first.");
        return;
    }

    // Initialize food database
    let mut food_db = FoodDatabase::new();
    if let Err(e) = food_db.load() {
        println!("Warning: Could not load food database: {}", e);
    }

    // Initial user selection
    println!("\nStarting user session");
    let mut selected_index = match select_user(users) {
        Some(index) => index,
        None => {
            println!("No user selected. Exiting session.");
            return;
        }
    };

    // Initialize food log for the selected user
    let mut food_log = FoodLog::new(&users[selected_index].name);
    if let Err(e) = food_log.load(&food_db) {
        println!("Warning: Could not load food log: {}", e);
    }

    println!("Selected user: {}", users[selected_index].name);
    
    loop {
        println!("\nUser Session - Current user: {}", users[selected_index].name);
        println!("1. List All Users");
        println!("2. Change Selected User");
        println!("3. Modify Current User");
        println!("4. Food Database Management");
        println!("5. Food Log Management");
        println!("6. View Statistics and Reports");
        println!("7. Exit Session");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        
        match input.trim() {
            "1" => {
                list_users(users);
            }
            "2" => {
                list_users(users);
                if let Some(index) = select_user(users) {
                    // Save current user's food log before switching
                    if let Err(e) = food_log.save() {
                        println!("Warning: Failed to save food log: {}", e);
                    }
                    
                    // Switch user and load their food log
                    selected_index = index;
                    food_log = FoodLog::new(&users[selected_index].name);
                    if let Err(e) = food_log.load(&food_db) {
                        println!("Warning: Could not load food log: {}", e);
                    }
                    
                    println!("Changed to user: {}", users[selected_index].name);
                } else {
                    println!("No change in selected user.");
                }
            }
            "3" => {
                modify_user(&mut users[selected_index]);
                println!("User modified successfully.");
            }
            "4" => {
                food_database_menu(&mut food_db);
            }
            "5" => {
                food_log_menu(&mut food_log, &food_db);
            }
            "6" => {
                statistics_menu(&food_log, &users[selected_index]);
            }
            "7" => {
                // Save food database and log before exiting
                if let Err(e) = food_db.save() {
                    println!("Warning: Failed to save food database: {}", e);
                }
                if let Err(e) = food_log.save() {
                    println!("Warning: Failed to save food log: {}", e);
                }
                println!("Exiting user session.");
                return;
            }
            _ => println!("Invalid option. Please try again."),
        }
    }
}

fn food_database_menu(food_db: &mut FoodDatabase) {

    loop {
        let mut choice = String::new();
        println!("Food Database Menu:");
        println!("1. Add a Basic Food Item");
        println!("2. Add a Composite Food Item");
        println!("3. Search Foods");
        println!("4. Add Food from Website");
        println!("5. Return to Main Menu");
        println!();
        println!("Enter your choice: ");
        std::io::stdin().read_line(&mut choice).unwrap();
        match choice.trim() {
            "1" => {
                // Get food details
                let mut name = String::new();
                println!("Enter the name of the food item: ");
                std::io::stdin().read_line(&mut name).unwrap();
                name = name.trim().to_string();
                
                // Get keywords
                let mut keywords_input = String::new();
                println!("Enter keywords separated by commas: ");
                std::io::stdin().read_line(&mut keywords_input).unwrap();
                let keywords: Vec<String> = keywords_input
                    .trim()
                    .split(',')
                    .map(|k| k.trim().to_string())
                    .collect();
                
                // Get calories
                let mut calories = String::new();
                println!("Enter the calories per serving: ");
                std::io::stdin().read_line(&mut calories).unwrap();
                let calories: f64 = match calories.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("Invalid input for calories, please try again.");
                        continue;
                    }
                };
                
                // Add to database
                match food_db.add_basic_food(&name, keywords, calories) {
                    Ok(_) => println!("Basic food '{}' added successfully.", name),
                    Err(e) => println!("Failed to add basic food: {}", e),
                }
            }
            "2" => {
                // Get composite food details
                let mut name = String::new();
                println!("Enter the name of the composite food item: ");
                std::io::stdin().read_line(&mut name).unwrap();
                name = name.trim().to_string();
                
                // Get keywords
                let mut keywords_input = String::new();
                println!("Enter keywords separated by commas: ");
                std::io::stdin().read_line(&mut keywords_input).unwrap();
                let keywords: Vec<String> = keywords_input
                    .trim()
                    .split(',')
                    .map(|k| k.trim().to_string())
                    .collect();
                
                // Add components
                let mut components: Vec<(String, f64)> = Vec::new();
                loop {
                    let mut component_name = String::new();
                    println!("Enter the name of the food component (or 'done' to finish): ");
                    std::io::stdin().read_line(&mut component_name).unwrap();
                    if component_name.trim() == "done" {
                        break;
                    }
                    
                    let search_term = component_name.trim();
                    let results = food_db.search_foods(search_term);
                    if results.is_empty() {
                        println!("No food item found with that name, please try again.");
                        continue;
                    }
                    
                    println!("Found food items:");
                    for (i, (name, calories)) in results.iter().enumerate() {
                        println!("{}. {} ({} calories)", i+1, name, calories);
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
                    
                    let mut quantity = String::new();
                    println!("Enter the quantity: ");
                    std::io::stdin().read_line(&mut quantity).unwrap();
                    let quantity: f64 = match quantity.trim().parse() {
                        Ok(num) => num,
                        Err(_) => {
                            println!("Invalid input for quantity, please try again.");
                            continue;
                        }
                    };
                    
                    components.push((selected_food.0.to_string(), quantity));
                }
                
                if components.is_empty() {
                    println!("Cannot create a composite food with no components.");
                    continue;
                }
                
                // Add to database
                match food_db.add_composite_food(&name, keywords, components) {
                    Ok(_) => println!("Composite food '{}' added successfully.", name),
                    Err(e) => println!("Failed to add composite food: {}", e),
                }
            }
            "3" => {
                // Search foods
                let mut search_term = String::new();
                println!("Enter search term: ");
                std::io::stdin().read_line(&mut search_term).unwrap();
                search_term = search_term.trim().to_string();
                
                let results = food_db.search_foods(&search_term);
                if results.is_empty() {
                    println!("No food items found matching '{}'", search_term);
                } else {
                    println!("Found food items:");
                    for (i, (name, calories)) in results.iter().enumerate() {
                        println!("{}. {} ({} calories)", i+1, name, calories);
                    }
                }
            }
            "4" => {
                // Add food from website
                add_food_from_website(food_db);
            }
            "5" => break,
            _ => println!("Invalid choice, please try again."),
        }
    }
}

fn add_food_from_website(food_db: &mut FoodDatabase) {
    // Get website URL from user
    let mut url = String::new();
    println!("Enter the website URL for the food information: ");
    std::io::stdin().read_line(&mut url).expect("Failed to read input");
    let mut url = url.trim().to_string();
    
    if url.is_empty() {
        println!("URL cannot be empty. Returning to menu.");
        return;
    }
    
    // Add https:// prefix if not present
    if !url.starts_with("http://") && !url.starts_with("https://") {
        url = format!("https://{}", url);
        println!("Added https:// prefix to URL: {}", url);
    }
    
    println!("Processing website at: {}", url);
    println!("This may take a few moments as we analyze the webpage...");
    
    // Create a tokio runtime to run the async function
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            println!("Failed to create runtime: {}", e);
            return;
        }
    };
    
    // Execute the async function within the runtime
    match rt.block_on(food_db.add_food_from_website_with_edit(&url)) {
        Ok(Some(food)) => {
            println!("Successfully added food '{}' with {} calories per serving.", 
                food.identifier, food.calories_per_serving);
        },
        Ok(None) => {
            println!("Food was not added to the database.");
        },
        Err(e) => {
            println!("Error adding food from website: {}", e);
            println!("Try again with a different URL or check if Ollama is running.");
        }
    }
}

fn food_log_menu(food_log: &mut FoodLog, food_db: &FoodDatabase) {
    loop {
        println!("\nFood Log Menu - Current Date: {}", food_log.current_date);
        println!("1. Add Food to Today's Log");
        println!("2. View Today's Log");
        println!("3. Change Current Date");
        println!("4. View Log for Specific Date");
        println!("5. Remove Food Entry");
        println!("6. Undo Last Action");
        println!("7. Return to User Session");

        let mut choice = String::new();
        println!("Enter your choice: ");
        io::stdin().read_line(&mut choice).expect("Failed to read input");

        match choice.trim() {
            "1" => {
                // Add food to log
                add_food_to_log(food_log, food_db);
            }
            "2" => {
                // View current log
                view_daily_log(food_log);
            }
            "3" => {
                // Change date
                change_log_date(food_log);
            }
            "4" => {
                // View log for specific date
                view_log_for_specific_date(food_log);
            }
            "5" => {
                // Remove food entry
                remove_food_from_log(food_log);
            }
            "6" => {
                // Undo last action
                match food_log.undo() {
                    Ok(_) => println!("Last action undone."),
                    Err(e) => println!("Could not undo: {}", e),
                }
            }
            "7" => break,
            _ => println!("Invalid choice, please try again."),
        }
    }
}

fn add_food_to_log(food_log: &mut FoodLog, food_db: &FoodDatabase) {
    // Search for food
    let mut search_term = String::new();
    println!("Enter food name to search: ");
    io::stdin().read_line(&mut search_term).expect("Failed to read input");
    search_term = search_term.trim().to_string();
    
    let results = food_db.search_foods(&search_term);
    if results.is_empty() {
        println!("No food items found matching '{}'", search_term);
        return;
    }
    
    // Display results
    println!("Found food items:");
    for (i, (name, calories)) in results.iter().enumerate() {
        println!("{}. {} ({} calories per serving)", i+1, name, calories);
    }
    
    // Select food
    println!("Enter the number of the food item (or 0 to cancel): ");
    let mut index = String::new();
    io::stdin().read_line(&mut index).expect("Failed to read input");
    let index: usize = match index.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid input. Returning to menu.");
            return;
        }
    };
    
    if index == 0 {
        return;
    } else if index > results.len() {
        println!("Invalid selection. Returning to menu.");
        return;
    }
    
    // Get the food
    let selected_food_id = results[index - 1].0;
    
    // Try to get as basic food first
    if let Some(food) = food_db.get_basic_food(selected_food_id) {
        // Get servings
        println!("Enter number of servings: ");
        let mut servings = String::new();
        io::stdin().read_line(&mut servings).expect("Failed to read input");
        let servings: f64 = match servings.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid input for servings. Returning to menu.");
                return;
            }
        };
        
        // Add to log
        if let Err(e) = food_log.add_food_entry(food, servings) {
            println!("Error adding food to log: {}", e);
        } else {
            println!("Added {} servings of {} to log.", servings, food.identifier);
        }
    } 
    // Check if it's a composite food
    else if let Some(composite_food) = food_db.get_composite_food(selected_food_id) {
        println!("This is a composite food item made of:");
        for (basic_food, quantity) in &composite_food.components {
            println!("- {} x{}", basic_food.identifier, quantity);
        }
        
        // Get servings
        println!("Enter number of servings: ");
        let mut servings = String::new();
        io::stdin().read_line(&mut servings).expect("Failed to read input");
        let servings: f64 = match servings.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid input for servings. Returning to menu.");
                return;
            }
        };
        
        // For composite foods, we'll add each component individually to the log
        println!("Adding components to log...");
        for (basic_food, quantity) in &composite_food.components {
            if let Err(e) = food_log.add_food_entry(basic_food, servings * quantity) {
                println!("Error adding {} to log: {}", basic_food.identifier, e);
            } else {
                println!("Added {} servings of {}", servings * quantity, basic_food.identifier);
            }
        }
        println!("Composite food '{}' added to log.", composite_food.identifier);
    } else {
        println!("Could not find the selected food item in the database.");
    }
}

// New function to view log for a specific date
fn view_log_for_specific_date(food_log: &FoodLog) {
    println!("Enter date to view (YYYY-MM-DD): ");
    let mut date = String::new();
    io::stdin().read_line(&mut date).expect("Failed to read input");
    let date = date.trim();
    
    if let Some(entries) = food_log.get_entries_for_date(date) {
        println!("\nFood Log for {}", date);
        
        if entries.is_empty() {
            println!("No entries for this date.");
            return;
        }
        
        let mut total_calories = 0.0;
        println!("Food Items:");
        println!("------------------------------------");
        for (i, entry) in entries.iter().enumerate() {
            let calories = entry.calories * entry.servings;
            println!("{}. {} (x{:.1} servings) - {:.1} calories", 
                i+1, entry.food_id, entry.servings, calories);
            total_calories += calories;
        }
        println!("------------------------------------");
        println!("Total Calories: {:.1}", total_calories);
        
        // Show comparison to target
        if let Some((_, target, difference)) = food_log.compare_to_target(date, &UserProfile{
            name: String::new(),
            height: 0.0,
            weight: 0.0,
            age: 0,
            gender: user_profile::Gender::Male,
            activity_level: user_profile::ActivityLevel::Sedentary,
            target_calorie_calc_strategy: user_profile::TargetCalorieCalcStrategy::MifflinStJeor,
            target_calorie: 0.0,
        }) {
            println!("Daily target: {:.1}", target);
            if difference > 0.0 {
                println!("You were {:.1} calories over your target.", difference);
            } else if difference < 0.0 {
                println!("You were {:.1} calories under your target.", -difference);
            } else {
                println!("You exactly met your target.");
            }
        }
    } else {
        println!("No log found for date: {}", date);
    }
}

fn view_daily_log(food_log: &FoodLog) {
    if let Some(daily_log) = food_log.get_current_log() {
        println!("\nFood Log for {}", daily_log.date);
        
        if daily_log.entries.is_empty() {
            println!("No entries for this date.");
            return;
        }
        
        let mut total_calories = 0.0;
        println!("Food Items:");
        println!("------------------------------------");
        for (i, entry) in daily_log.entries.iter().enumerate() {
            let calories = entry.calories * entry.servings;
            println!("{}. {} (x{:.1} servings) - {:.1} calories", 
                i+1, entry.food_id, entry.servings, calories);
            total_calories += calories;
        }
        println!("------------------------------------");
        println!("Total Calories: {:.1}", total_calories);
    } else {
        println!("No log found for the current date.");
    }
}

fn change_log_date(food_log: &mut FoodLog) {
    println!("Enter date (YYYY-MM-DD): ");
    let mut date = String::new();
    io::stdin().read_line(&mut date).expect("Failed to read input");
    
    match food_log.set_current_date(date.trim()) {
        Ok(_) => println!("Date changed to {}", date.trim()),
        Err(e) => println!("Error changing date: {}", e),
    }
}

fn remove_food_from_log(food_log: &mut FoodLog) {
    // First view the log so user can see what to remove
    view_daily_log(food_log);
    
    // Get the current log
    if let Some(daily_log) = food_log.get_current_log() {
        if daily_log.entries.is_empty() {
            return; // Nothing to remove
        }
        
        println!("Enter the number of the item to remove (or 0 to cancel): ");
        let mut index = String::new();
        io::stdin().read_line(&mut index).expect("Failed to read input");
        let index: usize = match index.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid input. Returning to menu.");
                return;
            }
        };
        
        if index == 0 {
            return;
        } else if index > daily_log.entries.len() {
            println!("Invalid selection.");
            return;
        }
        
        let food_id = daily_log.entries[index - 1].food_id.clone();
        
        match food_log.remove_food_entry(&food_id) {
            Ok(_) => println!("Food removed from log."),
            Err(e) => println!("Error removing food: {}", e),
        }
    }
}

fn statistics_menu(food_log: &FoodLog, user_profile: &UserProfile) {
    loop {
        println!("\nStatistics Menu");
        println!("1. View Today's Summary");
        println!("2. View Weekly Summary");
        println!("3. View Monthly Summary");
        println!("4. View Summary for Specific Date Range");
        println!("5. View All Logged Dates");
        println!("6. Return to User Session");

        let mut choice = String::new();
        println!("Enter your choice: ");
        io::stdin().read_line(&mut choice).expect("Failed to read input");

        match choice.trim() {
            "1" => {
                // View today's summary
                let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                view_date_summary(food_log, &today, user_profile);
            }
            "2" => {
                // View weekly summary (last 7 days)
                view_range_summary(food_log, 7, user_profile);
            }
            "3" => {
                // View monthly summary (last 30 days)
                view_range_summary(food_log, 30, user_profile);
            }
            "4" => {
                // View summary for specific date range
                custom_range_summary(food_log, user_profile);
            }
            "5" => {
                // View all logged dates
                let dates = food_log.get_logged_dates();
                if dates.is_empty() {
                    println!("No logged dates found.");
                } else {
                    println!("Dates with food logs:");
                    for date in dates {
                        println!("- {}", date);
                    }
                }
            }
            "6" => break,
            _ => println!("Invalid choice, please try again."),
        }
    }
}

fn view_date_summary(food_log: &FoodLog, date: &str, user_profile: &UserProfile) {
    println!("\nSummary for {}", date);
    
    if let Some((actual, target, difference)) = food_log.compare_to_target(date, user_profile) {
        println!("Total calories consumed: {:.1}", actual);
        println!("Daily target: {:.1}", target);
        println!("Difference: {:.1}", difference);
        
        if difference > 0.0 {
            println!("You are {:.1} calories over your target.", difference);
        } else if difference < 0.0 {
            println!("You are {:.1} calories under your target.", -difference);
        } else {
            println!("You have exactly met your target.");
        }
    } else {
        println!("No log data for this date.");
    }
}

fn view_range_summary(food_log: &FoodLog, days: i64, user_profile: &UserProfile) {
    let end_date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let start_date = chrono::Local::now()
        .checked_sub_signed(chrono::Duration::days(days - 1))
        .unwrap()
        .format("%Y-%m-%d")
        .to_string();
    
    match get_calorie_summary(food_log, &start_date, &end_date, user_profile) {
        Ok(summary) => display_summary_table(summary),
        Err(e) => println!("Error getting summary: {}", e),
    }
}

fn custom_range_summary(food_log: &FoodLog, user_profile: &UserProfile) {
    println!("Enter start date (YYYY-MM-DD): ");
    let mut start_date = String::new();
    io::stdin().read_line(&mut start_date).expect("Failed to read input");
    
    println!("Enter end date (YYYY-MM-DD): ");
    let mut end_date = String::new();
    io::stdin().read_line(&mut end_date).expect("Failed to read input");
    
    match get_calorie_summary(food_log, start_date.trim(), end_date.trim(), user_profile) {
        Ok(summary) => display_summary_table(summary),
        Err(e) => println!("Error getting summary: {}", e),
    }
}

fn display_summary_table(summary: Vec<(String, f64, f64, f64)>) {
    if summary.is_empty() {
        println!("No data available for the selected date range.");
        return;
    }
    
    println!("{:<12} {:>10} {:>10} {:>10}", "Date", "Actual", "Target", "Difference");
    println!("--------------------------------------------------");
    
    let mut total_actual = 0.0;
    let mut total_target = 0.0;
    
    for (date, actual, target, diff) in &summary {
        println!("{:<12} {:>10.1} {:>10.1} {:>10.1}", date, actual, target, diff);
        total_actual += actual;
        total_target += target;
    }
    
    println!("--------------------------------------------------");
    let avg_actual = total_actual / summary.len() as f64;
    let avg_target = total_target / summary.len() as f64;
    let avg_diff = avg_actual - avg_target;
    
    println!("{:<12} {:>10.1} {:>10.1} {:>10.1}", 
        "Average", avg_actual, avg_target, avg_diff);
    println!("{:<12} {:>10.1} {:>10.1} {:>10.1}", 
        "Total", total_actual, total_target, total_actual - total_target);
}