use crate::food_database::BasicFood;

#[derive(Debug)]
pub struct DailyLog {
    pub date: String,
    pub entries: Vec<LogEntry>,
}

impl DailyLog {
    pub fn new() -> Self {
        Self {
            date: "2023-01-01".to_string(),
            entries: Vec::new(),
        }
    }

    pub fn add_food_entry(&mut self, food: BasicFood, servings: f64) {
        self.entries.push(LogEntry { food, servings });
    }

    pub fn calculate_total_calories(&self) -> f64 {
        self.entries
            .iter()
            .map(|entry| entry.food.get_calories() * entry.servings)
            .sum()
    }
}

#[derive(Debug)]
pub struct LogEntry {
    pub food: BasicFood,
    pub servings: f64,
}
