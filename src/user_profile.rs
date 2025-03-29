#[derive(Debug)]
pub struct UserProfile {
    pub gender: String,
    pub height: f64,
    pub weight: f64,
    pub age: u32,
    pub activity_level: String,
}

impl UserProfile {
    pub fn new(gender: &str, height: f64, weight: f64, age: u32, activity_level: &str) -> Self {
        Self {
            gender: gender.to_string(),
            height,
            weight,
            age,
            activity_level: activity_level.to_string(),
        }
    }

    pub fn update_profile(
        &mut self,
        height: Option<f64>,
        weight: Option<f64>,
        age: Option<u32>,
        activity_level: Option<&str>,
    ) {
        if let Some(h) = height {
            self.height = h;
        }
        if let Some(w) = weight {
            self.weight = w;
        }
        if let Some(a) = age {
            self.age = a;
        }
        if let Some(al) = activity_level {
            self.activity_level = al.to_string();
        }
    }
}
