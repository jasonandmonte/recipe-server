-- Add up migration script here
CREATE TABLE recipes (
  id VARCHAR(200) UNIQUE PRIMARY KEY NOT NULL,
  title VARCHAR(200) NOT NULL,
  ingredients TEXT NOT NULL,
  instructions TEXT NOT NULL,
  recipe_source VARCHAR(200) NOT NULL
);

CREATE TABLE IF NOT EXISTS tags (
  recipe_id VARCHAR(200) NOT NULL,
  tag VARCHAR(200) NOT NULL,
  FOREIGN KEY (recipe_id) REFERENCES recipes(id)
);
