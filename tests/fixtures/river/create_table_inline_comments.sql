CREATE TABLE personnes (
    id INTEGER PRIMARY KEY, -- un type SERIAL aurait été bienvenu
    nom VARCHAR(100), -- nom de la colonne et type
    birthdate DATE CHECK (birthdate < CURRENT_DATE), --contrainte de colonne
    is_active BOOLEAN,
    UNIQUE (nom, birthdate) --contrainte de table
);
