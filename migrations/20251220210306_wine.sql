CREATE TABLE wines (
  wine_id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  year INT NOT NULL,
  image BLOB,
  UNIQUE (name,year)
);

CREATE TABLE wine_grapes (
  wine_id INTEGER NOT NULL,
  grape_name TEXT NOT NULL,
  FOREIGN KEY (wine_id) REFERENCES wines(wine_id),
  FOREIGN KEY (grape_name) REFERENCES grapes(name),
  PRIMARY KEY(wine_id, grape_name)
);

CREATE TABLE wine_inventory_events (
  wine_id INTEGER NOT NULL,
  dt DATETIME NOT NULL,
  bottles INT NOT NULL,
  FOREIGN KEY (wine_id) REFERENCES wines(wine_id)
);

CREATE TABLE wine_comments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  wine_id INTEGER NOT NULL,
  dt DATETIME NOT NULL,
  comment TEXT NOT NULL,
  FOREIGN KEY (wine_id) REFERENCES wines(wine_id)
)

