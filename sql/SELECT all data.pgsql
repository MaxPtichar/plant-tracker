SELECT 
    p.id AS plant_id,
    p.plants_name,
    p.plant_type,
    p.light_level,
    p.avg_r,
    pc.pot_weight,
    pc.dry_soil_weight,
    pc.pot_diameter_cm,
    pc.soil_type,
    m.weight AS measurement_weight,
    m.date AS measurement_date,
    m.measuring_type
FROM plants p
-- Присоединяем конфигурацию горшка (обычно 1 к 1 для активного растения)
LEFT JOIN pot_configs pc ON p.id = pc.plant_id
-- Присоединяем все измерения (1 ко многим)
LEFT JOIN measurements m ON p.id = m.plant_id
ORDER BY m.date DESC;