CREATE TABLE wine_food_pairings (
  id      INTEGER PRIMARY KEY AUTOINCREMENT,
  wine_id INTEGER NOT NULL,
  food    TEXT    NOT NULL COLLATE NOCASE,
  FOREIGN KEY (wine_id) REFERENCES wines(wine_id),
  UNIQUE (wine_id, food)
);

CREATE INDEX wine_food_pairings_wine_id ON wine_food_pairings(wine_id);
CREATE INDEX wine_food_pairings_food    ON wine_food_pairings(food COLLATE NOCASE);
