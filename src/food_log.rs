use std::fs;
use std::io;
use std::path::Path;
use std::collections::HashMap;
use chrono::{Local, NaiveDate};
use serde::{Serialize, Deserialize};

use crate::food_database::{FoodDatabase, BasicFood};
use crate::user_profile::UserProfile;

// Struct to handle food logging for a specific user
#[derive(Debug)]
pub struct FoodLog {
    user_name: String, // Used for file naming
    daily_logs: HashMap<String, DailyLog>,
    pub current_date: String, // Make this public so we can access it from main
    log_dir_path: String,
}

// A single day's log entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyLog {
    pub date: String,
    pub entries: Vec<LogEntry>,
    #[serde(skip)]
    undo_stack: Vec<UndoAction>, // Not serialized
}

// Represents a single food entry in the log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub food_id: String,  // ID/name of the food
    pub servings: f64,    // Number of servings
    pub calories: f64,    // Pre-calculated calories
}

// Action type for undo feature
#[derive(Debug, Clone)]
enum UndoAction {
    Add(String, f64),    // (food_id, previous_servings) - Updated to match usage
    Remove(LogEntry),     // Removed entry to restore
}

// Serialization format for the entire log file
#[derive(Serialize, Deserialize)]
struct SerializedFoodLog {
    user_name: String,
    daily_logs: Vec<DailyLog>,
}

impl FoodLog {
    pub fn new(user_name: &str) -> Self {
        let today = Local::now().format("%Y-%m-%d").to_string();
        
        Self {
            user_name: user_name.to_string(),
            daily_logs: HashMap::new(),
            current_date: today,
            log_dir_path: "data/logs".to_string(),
        }
    }

    // Load logs for the specified user
    pub fn load(&mut self, _food_db: &FoodDatabase) -> Result<(), io::Error> {
        let log_path = format!("{}/{}_logs.yaml", self.log_dir_path, self.user_name);
        
        if Path::new(&log_path).exists() {
            let contents = fs::read_to_string(&log_path)?;
            let serialized_log: SerializedFoodLog = serde_yaml::from_str(&contents)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            
            // Clear existing logs and reset with loaded data
            self.daily_logs.clear();
            
            for log in serialized_log.daily_logs {
                let daily_log = DailyLog {
                    date: log.date.clone(),
                    entries: log.entries,
                    undo_stack: Vec::new(),
                };
                
                self.daily_logs.insert(log.date, daily_log);
            }
        }
        
        Ok(())
    }

    // Save logs for the current user
    pub fn save(&self) -> Result<(), io::Error> {
        // Ensure log directory exists
        if !Path::new(&self.log_dir_path).exists() {
            fs::create_dir_all(&self.log_dir_path)?;
        }
        
        // Convert HashMap to Vec for serialization
        let logs_vec: Vec<DailyLog> = self.daily_logs.values().cloned().collect();
        
        let serialized_log = SerializedFoodLog {
            user_name: self.user_name.clone(),
            daily_logs: logs_vec,
        };
        
        let yaml = serde_yaml::to_string(&serialized_log)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let log_path = format!("{}/{}_logs.yaml", self.log_dir_path, self.user_name);
        fs::write(&log_path, yaml)?;
        
        Ok(())
    }
    
    // Change the current date for logging
    pub fn set_current_date(&mut self, date: &str) -> Result<(), io::Error> {
        // Validate date format
        if NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid date format. Use YYYY-MM-DD."
            ));
        }
        
        self.current_date = date.to_string();
        
        // Ensure we have a log for this date
        if !self.daily_logs.contains_key(date) {
            self.daily_logs.insert(date.to_string(), DailyLog::new(date));
        }
        
        Ok(())
    }
    
    // Add food entry to the current date's log
    pub fn add_food_entry(&mut self, food: &BasicFood, servings: f64) -> Result<(), io::Error> {
        // Get or create log for current date
        let daily_log = self.daily_logs
            .entry(self.current_date.clone())
            .or_insert_with(|| DailyLog::new(&self.current_date));
        
        daily_log.add_entry(food, servings);
        
        // Save after each modification
        self.save()?;
        
        Ok(())
    }
    
    // Remove food entry from the current date's log
    pub fn remove_food_entry(&mut self, food_id: &str) -> Result<(), io::Error> {
        if let Some(daily_log) = self.daily_logs.get_mut(&self.current_date) {
            daily_log.remove_entry(food_id)?;
            self.save()?;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("No log found for date: {}", self.current_date)
            ))
        }
    }
    
    // Undo last action for current date's log
    pub fn undo(&mut self) -> Result<(), io::Error> {
        if let Some(daily_log) = self.daily_logs.get_mut(&self.current_date) {
            daily_log.undo()?;
            self.save()?;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("No log found for date: {}", self.current_date)
            ))
        }
    }
    
    // Get the entries for a specific date
    pub fn get_entries_for_date(&self, date: &str) -> Option<&Vec<LogEntry>> {
        self.daily_logs.get(date).map(|log| &log.entries)
    }
    
    // Get the current date's log
    pub fn get_current_log(&self) -> Option<&DailyLog> {
        self.daily_logs.get(&self.current_date)
    }
    
    // Calculate total calories for a specific date
    pub fn calculate_calories_for_date(&self, date: &str) -> f64 {
        if let Some(daily_log) = self.daily_logs.get(date) {
            daily_log.calculate_total_calories()
        } else {
            0.0
        }
    }
    
    // Get all dates that have logs
    pub fn get_logged_dates(&self) -> Vec<String> {
        self.daily_logs.keys().cloned().collect()
    }
    
    // Compares actual calorie intake to target for a given date
    pub fn compare_to_target(&self, date: &str, user_profile: &UserProfile) -> Option<(f64, f64, f64)> {
        if let Some(daily_log) = self.daily_logs.get(date) {
            let actual = daily_log.calculate_total_calories();
            let target = user_profile.target_calorie;
            let difference = actual - target;
            
            Some((actual, target, difference))
        } else {
            None
        }
    }
}

impl DailyLog {
    pub fn new(date: &str) -> Self {
        Self {
            date: date.to_string(),
            entries: Vec::new(),
            undo_stack: Vec::new(),
        }
    }
    
    // Add a food entry to this day's log, updating servings if it already exists
    pub fn add_entry(&mut self, food: &BasicFood, servings: f64) {
        // Check if this food already exists in today's entries
        if let Some(existing_entry) = self.entries.iter_mut()
            .find(|e| e.food_id == food.identifier) {
            
            // Store previous servings for undo
            let prev_servings = existing_entry.servings;
            self.undo_stack.push(UndoAction::Add(food.identifier.clone(), prev_servings));
            
            // Update the servings
            existing_entry.servings += servings;
        } else {
            // If food doesn't exist yet, create a new entry
            let entry = LogEntry {
                food_id: food.identifier.clone(),
                servings,
                calories: food.calories_per_serving,
            };
            
            self.entries.push(entry);
            
            // Add undo action with 0 as previous servings (new item)
            self.undo_stack.push(UndoAction::Add(food.identifier.clone(), 0.0));
        }
    }
    
    // Remove a food entry from this day's log by food_id
    pub fn remove_entry(&mut self, food_id: &str) -> Result<(), io::Error> {
        if let Some(pos) = self.entries.iter().position(|e| e.food_id == food_id) {
            // Store the entry for potential undo
            let removed_entry = self.entries.remove(pos);
            self.undo_stack.push(UndoAction::Remove(removed_entry));
            
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Food '{}' not found in log", food_id)
            ))
        }
    }
    
    // Updated undo function to work with new action types
    pub fn undo(&mut self) -> Result<(), io::Error> {
        if let Some(action) = self.undo_stack.pop() {
            match action {
                UndoAction::Add(food_id, prev_servings) => {
                    if let Some(entry) = self.entries.iter_mut().find(|e| e.food_id == food_id) {
                        if prev_servings > 0.0 {
                            // Was an update, restore previous servings
                            entry.servings = prev_servings;
                        } else {
                            // Was a new entry, remove it
                            if let Some(pos) = self.entries.iter().position(|e| e.food_id == food_id) {
                                self.entries.remove(pos);
                            }
                        }
                        Ok(())
                    } else {
                        Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid undo action: food not found"
                        ))
                    }
                },
                UndoAction::Remove(entry) => {
                    self.entries.push(entry);
                    Ok(())
                }
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No actions to undo"
            ))
        }
    }
    
    // Calculate the total calories for this day
    pub fn calculate_total_calories(&self) -> f64 {
        self.entries.iter().map(|e| e.calories * e.servings).sum()
    }
}

// Utility functions for food logs

// Get summary statistics for a date range
pub fn get_calorie_summary(
    food_log: &FoodLog, 
    start_date: &str, 
    end_date: &str,
    user_profile: &UserProfile
) -> Result<Vec<(String, f64, f64, f64)>, io::Error> {
    // Validate date format
    if NaiveDate::parse_from_str(start_date, "%Y-%m-%d").is_err() ||
       NaiveDate::parse_from_str(end_date, "%Y-%m-%d").is_err() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid date format. Use YYYY-MM-DD."
        ));
    }
    
    let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
        .expect("Date format already validated");
    let end = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        .expect("Date format already validated");
    
    if start > end {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Start date cannot be after end date"
        ));
    }
    
    let mut results = Vec::new();
    let mut current = start;
    let target = user_profile.target_calorie;
    
    while current <= end {
        let current_str = current.format("%Y-%m-%d").to_string();
        let actual = food_log.calculate_calories_for_date(&current_str);
        let difference = actual - target;
        
        results.push((current_str, actual, target, difference));
        
        current = current.succ_opt().unwrap(); // Move to next day
    }
    
    Ok(results)
}