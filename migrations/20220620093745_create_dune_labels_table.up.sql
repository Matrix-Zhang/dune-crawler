-- Add up migration script here
CREATE TABLE IF NOT EXISTS dune_labels
(
    id         SERIAL PRIMARY KEY,
    address    CHAR(42) NOT NULL,
    label_type TEXT     NOT NULL,
    label_name TEXT     NOT NULL
);
