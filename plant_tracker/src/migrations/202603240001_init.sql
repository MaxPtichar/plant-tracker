-- 1. сначала дропаем
DROP TABLE IF EXISTS measurements CASCADE;
DROP TABLE IF EXISTS pot_configs CASCADE;
DROP TABLE IF EXISTS plants CASCADE;
DROP TABLE IF EXISTS users CASCADE;

-- 2. создаём с новыми полями сразу
CREATE TABLE users (
    id BIGINT PRIMARY KEY,
    username TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP 
);

CREATE TABLE plants (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plants_name TEXT NOT NULL,
    plant_type TEXT NOT NULL DEFAULT 'Regular' 
        CHECK (plant_type IN ('Regular', 'Succulent', 'Tropical')),
    light_level TEXT NOT NULL DEFAULT 'window'
        CHECK (light_level IN ('window', 'shadow')),
    air_circulation TEXT NOT NULL DEFAULT 'normal'
        CHECK (air_circulation IN ('normal', 'stagnant')),
    transpiration_coef REAL NOT NULL DEFAULT 1.0,
    avg_r REAL DEFAULT 0.0,
    cycles_count INT NOT NULL DEFAULT 0
);

CREATE TABLE pot_configs (
    id BIGSERIAL PRIMARY KEY,
    plant_id BIGINT NOT NULL REFERENCES plants(id) ON DELETE CASCADE,
    pot_weight BIGINT NOT NULL,
    dry_soil_weight BIGINT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    pot_diameter_cm REAL NOT NULL DEFAULT 16.0,
    soil_type TEXT NOT NULL DEFAULT 'universal' 
        CHECK (soil_type IN ('universal', 'succulent', 'tropical'))
);

CREATE TABLE measurements (
    id BIGSERIAL PRIMARY KEY,
    plant_id BIGINT NOT NULL REFERENCES plants(id) ON DELETE CASCADE, 
    weight REAL NOT NULL,
    date TIMESTAMPTZ NOT NULL,
    measuring_type TEXT NOT NULL
);