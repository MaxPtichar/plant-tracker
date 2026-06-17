

ALTER TABLE watering_config ADD COLUMN learned_daily_loss REAL;


DROP TABLE IF EXISTS measurements CASCADE;
DROP TABLE IF EXISTS pot_configs CASCADE;
DROP TABLE IF EXISTS plants CASCADE;
DROP TABLE IF EXISTS users CASCADE;
-- 2. создаём с новыми полями сразу
CREATE TABLE IF NOT EXISTS  users (
    id BIGINT PRIMARY KEY,
    username TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS   plants (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plants_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS  watering_config (
    id BIGSERIAL PRIMARY KEY,
    plant_id BIGINT NOT NULL REFERENCES plants(id) ON DELETE CASCADE,
    wet_weight BIGINT NOT NULL,
    dry_weight BIGINT NOT NULL,
    threshold_pct REAL NOT NULL DEFAULT 0.30,       
    learned_threshold_pct REAL,                      
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS  measurements (
    id BIGSERIAL PRIMARY KEY,
    plant_id BIGINT NOT NULL REFERENCES plants(id) ON DELETE CASCADE, 
    weight REAL NOT NULL,
    date TIMESTAMPTZ NOT NULL,
    measuring_type TEXT NOT NULL
);