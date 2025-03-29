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
