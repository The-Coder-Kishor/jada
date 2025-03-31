CREATE TABLE BasicFood (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    calories_per_serving DOUBLE NOT NULL
);

CREATE TABLE CompositeFood (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL
);

CREATE TABLE CompositeFoodComponents (
    id INT AUTO_INCREMENT PRIMARY KEY,
    composite_food_id INT NOT NULL,
    basic_food_id INT NOT NULL,
    quantity DOUBLE NOT NULL,
    FOREIGN KEY (composite_food_id) REFERENCES CompositeFood(id),
    FOREIGN KEY (basic_food_id) REFERENCES BasicFood(id)
);

CREATE TABLE UserProfile (
    id INT AUTO_INCREMENT PRIMARY KEY,
    gender VARCHAR(10) NOT NULL,
    height DOUBLE NOT NULL,
    weight DOUBLE NOT NULL,
    age INT NOT NULL,
    activity_level VARCHAR(20) NOT NULL
);