DROP TABLE IF EXISTS pot_configs;
DROP TABLE IF EXISTS measurements;
DROP TABLE IF EXISTS plants;
DROP TABLE IF EXISTS users;



CREATE TABLE users (
    id BIGINT PRIMARY KEY,
    username TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP 
);

CREATE TABLE plants (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plants_name TEXT NOT NULL
);



CREATE TABLE pot_configs (
    id BIGSERIAL PRIMARY KEY,
    plant_id BIGINT NOT NULL REFERENCES plants(id),
    pot_weight BIGINT NOT NULL ,
    dry_soil_weight BIGINT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);


CREATE TABLE measurements (
    id BIGSERIAL PRIMARY KEY,
    plant_id BIGINT NOT NULL REFERENCES plants(id) ON DELETE CASCADE, 
    weight REAL NOT NULL,
    date DATE Not NULL,
    measuring_type TEXT NOT NULL
);


