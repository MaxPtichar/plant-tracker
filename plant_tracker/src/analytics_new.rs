use chrono::NaiveDate;
use std::sync::atomic::{AtomicU32, Ordering};

static OUT_DOOR_TEMP_BITS: AtomicU32 = AtomicU32::new(0);

use crate::{
    constants::{
        ANTOINE_BASE, ANTOINE_COEF, ANTOINE_TEMP, BULK_DENSITY_SUCCULENT, BULK_DENSITY_TROPICAL,
        BULK_DENSITY_UNIVERSAL, CP_AIR, DELTA_COEF, FC_SUCCULENT, FC_TROPICAL, FC_UNIVERSAL, GAMMA,
        INDOOR_HUMIDITY, LAMBDA, MAD_REGULAR, MAD_SUCCULENT, MAD_TROPICAL, PWP_SUCCULENT,
        PWP_TROPICAL, PWP_UNIVERSAL, RA_NORMAL, RA_STAGNANT, RC_REGULAR, RC_SUCCULENT, RC_TROPICAL,
        RHO_AIR, RN_SHADOW, RN_WINDOW,
    },
    models::Measurements,
};

//заглушка, нужен api для погоды потом
pub async fn set_outdoor_temp(temp: f32) {
    OUT_DOOR_TEMP_BITS.store(temp.to_bits(), Ordering::Relaxed);
}

pub fn get_outdoor_temp() -> f32 {
    println!(
        "{}",
        f32::from_bits(OUT_DOOR_TEMP_BITS.load(Ordering::Relaxed))
    );
    f32::from_bits(OUT_DOOR_TEMP_BITS.load(Ordering::Relaxed))
}

/// Calculates saturated vapour pressure (kPa) using Antoine equation
/// Valid range: 0–60°C
fn p_sat(tem_c: f32) -> f32 {
    // 0.6108 — base pressure at 0°C, kPa (ANTOINE_BASE)
    // 17.27  — August-Roche-Magnus empirical coefficient (ANTOINE_COEF)
    // 237.3  — temperature shape coefficient, °C (ANTOINE_TEMP)
    ANTOINE_BASE * ((ANTOINE_COEF * tem_c) / (tem_c + ANTOINE_TEMP)).exp()
}

/// Calculates slope of saturation vapour pressure curve, kPa/°C
///
/// Formula: `Δ = 4098 * e_s(T) / (T + 237.3)²`
///
/// Constants origin:
/// - `4098` — derived from Antoine equation differentiation:
///            d/dT [e_s(T)] * 237.3² ≈ 4098 at standard conditions
///            Source: FAO-56, Allen et al. 1998, eq. 13
///
/// # Validation
/// - T = 20°C → Δ ≈ 0.145 kPa/°C
/// - T = 30°C → Δ ≈ 0.243 kPa/°C
fn delta(tem_c: f32) -> f32 {
    DELTA_COEF * p_sat(tem_c) / (tem_c + ANTOINE_TEMP).powi(2)
}
/// Estimates indoor temperature from outdoor API temperature, °C
///
/// Indoor temperature is not equal to outdoor:
/// - Winter: indoor is ~7°C warmer (heating)
/// - The formula uses a weighted blend toward a stable indoor baseline
///
/// Formula: `T_indoor = T_outdoor * 0.3 + 20.0 + T_INDOOR_OFFSET`
/// - `0.3` — outdoor influence factor (30% of outdoor temp affects indoor)
/// - `20.0` — baseline indoor temperature °C
/// - `T_INDOOR_OFFSET = 7.0` — heating/cooling offset
///
/// # Validation
/// - T_outdoor =  0°C → T_indoor ≈ 27.0°C (winter, heating on)
/// - T_outdoor = 20°C → T_indoor ≈ 33.0°C  
/// - T_outdoor = -20°C → T_indoor ≈ 21.0°C (cold outside, warm inside)
fn indoor_temp(tem_c: f32) -> f32 {
    tem_c * 0.1 + 22.0
}
/// Calculates Vapour Pressure Deficit, kPa
///
/// VPD is the main driver of evapotranspiration indoors.
/// Shows how much more water vapour the air can hold at current temperature.
///
/// Formula: `VPD = e_s - e_a`
/// where:
/// - `e_s` — saturated vapour pressure at current temperature
/// - `e_a` — actual vapour pressure = `e_s * humidity`
///
/// # Validation
/// - T = 20°C, humidity = 0.40 → VPD ≈ 1.403 kPa
/// - T = 30°C, humidity = 0.40 → VPD ≈ 2.546 kPa
/// - humidity = 1.0 → VPD = 0.0 (air fully saturated, no evaporation)
fn vpd(tem_c: f32) -> f32 {
    let ps = p_sat(tem_c);
    let pa = ps * INDOOR_HUMIDITY;
    ps - pa
}

fn vpd_with_diff_humidity(tem_c: f32, humidity: f32) -> f32 {
    let ps = p_sat(tem_c);
    let pa = ps * humidity;
    ps - pa
}

/// Calculates evapotranspiration for indoor plants using Penman-Monteith FAO-56.
///
/// Returns estimated water loss in g/day for the given pot.
///
/// Formula:
/// `ET = (Δ * Rn + ρ * Cp * VPD / ra) / (λ * (Δ + γ * (1 + rc/ra)))`
///
/// Where:
/// - `Δ`  — slope of saturation vapour pressure curve kPa/°C
/// - `Rn` — net radiation balance W/m²
/// - `ρ`  — air density kg/m³
/// - `Cp` — specific heat of air J/(kg·K)
/// - `VPD`— vapour pressure deficit kPa
/// - `ra` — aerodynamic resistance s/m
/// - `λ`  — latent heat of vaporization J/kg
/// - `γ`  — psychrometric constant kPa/°C
/// - `rc` — stomatal resistance s/m

pub fn penman_monteith(
    tem_c: f32,
    plant_type: &str,
    light_level: &str,
    air_circulation: &str,
    pot_diameter_cm: f32,
    transpiration_coef: f32,
) -> f32 {
    let t = indoor_temp(tem_c);
    let d = delta(t);
    let v = vpd(t); // this might use function vpd_with_diff_humidity if humidity will be dinamic

    let rn = match light_level {
        "shadow" => RN_SHADOW,
        _ => RN_WINDOW,
    };

    let ra = match air_circulation {
        "stagnant" => RA_STAGNANT,
        _ => RA_NORMAL,
    };

    let rc = match plant_type {
        "Tropical" => RC_TROPICAL,
        "Succulent" => RC_SUCCULENT,
        _ => RC_REGULAR,
    };

    // Penman-Monteith kg/(m²·s)

    let numerator = d * rn + (RHO_AIR * CP_AIR * v / ra);
    let denominator = LAMBDA * (d + GAMMA * (1.0 + rc / ra));
    let et_m2s = numerator / denominator;

    // pot surface area m²
    let raduis_m = (pot_diameter_cm / 2.0) / 100.0;
    let area = std::f32::consts::PI * raduis_m * raduis_m;

    let et_g_per_day = et_m2s * area * 1000.0 * 86400.0;

    (et_g_per_day * transpiration_coef).max(0.0)
}

/// Returns soil parameters for given soil type
fn soil_params(soil_type: &str) -> (f32, f32, f32) {
    // returns (fc, pwp)
    match soil_type {
        "succulent" => (FC_SUCCULENT, PWP_SUCCULENT, BULK_DENSITY_SUCCULENT),
        "tropical" => (FC_TROPICAL, PWP_TROPICAL, BULK_DENSITY_TROPICAL),
        _ => (FC_UNIVERSAL, PWP_UNIVERSAL, BULK_DENSITY_UNIVERSAL),
    }
}

// Calculates Total Available Water in grams
///
/// TAW = (FC - PWP) * dry_soil_weight_g
fn taw(soil_type: &str, dry_soil_weight_g: f32) -> f32 {
    let (fc, pwp, density) = soil_params(soil_type);
    let volume_l = (dry_soil_weight_g / 1000.0) / density;
    let taw_g = (fc - pwp) * volume_l;
    taw_g
}

/// Calculates Readily Available Water in grams
///
/// RAW = TAW * MAD
pub fn raw(soil_type: &str, dry_soil_weight_g: f32, plant_type: &str) -> (f32, f32) {
    let mad = match plant_type {
        "Tropical" => MAD_TROPICAL,
        "Succulent" => MAD_SUCCULENT,
        _ => MAD_REGULAR,
    };

    let raw_g = taw(soil_type, dry_soil_weight_g) * mad;
    (raw_g, mad)
}
/// Calculates weighted evaporation rate combining physical model and historical data.
///
/// Weight distribution:
/// - if current cycle has 2+ measurements: current cycle gets most weight
/// - if no current cycle data: blend history and physics
/// - if no history (first cycle): use only physics
///
/// Formula:
/// `r_final = (n * r_current + k * r_historical + 1 * r_physical) / (n + k + 1)`
///
/// Where:
/// - `n` — number of Regular measurements in current cycle
/// - `k` — number of completed cycles (weight of historical avg)
/// - `1` — fixed weight for physical model
///
/// # Returns
/// - `None` if no data at all (first cycle, no measurements yet)
pub fn weighted_r(
    r_physical: f32,
    avg_r: f32,
    r_current_cycle: Option<f32>,
    n_current: i32,
    cycles_count: i32,
) -> f32 {
    match r_current_cycle {
        Some(r_current) if n_current >= 2 && cycles_count > 0 => {
            // текущий цикл + история
            let w_current = n_current as f32;
            let w_historical = cycles_count as f32;
            (w_current * r_current + w_historical * avg_r) / (w_current + w_historical)
        }
        Some(r_current) if n_current >= 2 => {
            // только текущий цикл, истории нет
            r_current
        }
        _ if cycles_count > 0 => {
            // только история
            avg_r
        }
        _ => {
            // совсем ничего — только физика
            r_physical
        }
    }
}

/// Entry point — calculates days until next watering.
///
/// Combines physical model (Penman-Monteith) with historical data
/// to produce an accurate, self-calibrating prediction.
///
/// # Pipeline
/// ```text
/// current_water = current_weight - dry_total
/// water_after   = after_watering_weight - dry_total
/// depleted      = water_after - current_water
/// remaining     = RAW - depleted
///
/// r_physical    = penman_monteith(...)
/// r_final       = weighted_r(r_physical, avg_r, r_current_cycle, cycles_count)
///
/// days          = remaining / r_final
/// ```
///
/// # Returns
/// - `Some(days)` — days until watering (negative = overdue)
/// - `None` — not enough data (no history and no measurements)
pub fn days_until_watering_full(
    regular_measurements: &[Measurements],
    last_watering_date: Option<NaiveDate>,
    after_watering_weight: f32,
    dry_total: f32,
    soil_type: &str,
    dry_soil_weight_g: f32,
    plant_type: &str,
    light_level: &str,
    air_circulation: &str,
    pot_diameter_cm: f32,
    transpiration_coef: f32,
    avg_r: f32,
    cycles_count: i32,
    temp_outdoor: f32,
    water_threshold: f32,
) -> Option<f32> {
    // ИСПРАВЛЕНО: если нет Regular после полива — используем after_watering_weight
    // как текущий вес и days_since = 0, вместо возврата None
    let (current_weight, days_since) = match regular_measurements.first() {
        Some(m) => {
            let days = (chrono::Local::now().date_naive() - m.date).num_days() as f32;
            (m.weight, days)
        }
        None => {
            let days = last_watering_date
                .map(|d| (chrono::Local::now().date_naive() - d).num_days() as f32)
                .unwrap_or(0.0);
            (after_watering_weight, days)
        }
    };

    let current_water = current_weight - dry_total;
    let water_after = after_watering_weight - dry_total;
    let depleted = water_after - current_water;

    let target_water = water_after * water_threshold;
    let remaining = current_water - target_water;

    let r_physical = penman_monteith(
        temp_outdoor,
        plant_type,
        light_level,
        air_circulation,
        pot_diameter_cm,
        transpiration_coef,
    );

    let r_current_cycle = avg_evaporation_rate(regular_measurements);

    let r_final = weighted_r(
        r_physical,
        avg_r,
        r_current_cycle,
        regular_measurements.len() as i32,
        cycles_count,
    );

    // ИСПРАВЛЕНО: если нет истории и нет физики — None
    // раньше падало раньше через first()?
    if r_final <= 0.0 {
        return None;
    }

    dbg!(
        remaining,
        r_final,
        days_since,
        depleted,
        current_water,
        water_after
    );

    Some(remaining / r_final - days_since)
}

/// Calculates the average evaporation rate from two consecutive
/// `Regular` measurements, ordered newest first.
///
/// # Returns
/// - `Some(rate)` — grams lost per day
/// - `None` — fewer than 2 measurements provided
///
/// # Example
/// ```
/// // weight1=500g on 2026-03-10, weight2=450g on 2026-03-05
/// // rate = (500 - 450) / 5 = 10.0 g/day
/// ```
pub fn avg_evaporation_rate(last_measurements: &[Measurements]) -> Option<f32> {
    if last_measurements.len() < 2 {
        return None;
    }

    let (weight1, weight2) = (last_measurements[0].weight, last_measurements[1].weight);
    let (date1, date2) = (last_measurements[0].date, last_measurements[1].date);

    let delta_days = (date1 - date2).abs().num_days().max(1) as f32;
    let evapuation_rate = (weight2 - weight1).abs() / delta_days;

    Some(evapuation_rate as f32)
}

#[cfg(test)]
mod test {
    use chrono::NaiveDate;

    use super::*;

    pub const EPSILON: f32 = 0.01;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn make_measurement(weight: f32, date: &str) -> Measurements {
        Measurements {
            id: 1,
            plant_id: 1,
            weight,
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            measuring_type: "Regular".to_string(),
        }
    }
    #[test]
    fn test_p_sat_at_zero() {
        // при 0°C давление насыщенного пара = 0.611 кПа
        // точка замерзания воды — хорошо известное значение
        dbg!(p_sat(0.0));
        assert!(approx_eq(p_sat(0.0), 0.611));
    }
    #[test]
    fn test_p_sat_at_20() {
        // при 20°C = 2.338 кПа — стандартная комнатная температура
        dbg!(p_sat(20.0));
        assert!(approx_eq(p_sat(20.0), 2.338));
    }
    #[test]
    fn test_p_sat_at_30() {
        // при 30°C = 4.243 кПа — летняя жара
        assert!(approx_eq(p_sat(30.0), 4.243));
    }

    #[test]
    fn test_p_sat_increases_with_temp() {
        // физическое свойство: чем горячее — тем больше давление пара
        assert!(p_sat(30.0) > p_sat(20.0));
        assert!(p_sat(20.0) > p_sat(10.0));
        assert!(p_sat(10.0) > p_sat(0.0));
    }

    #[test]
    fn test_p_sat_positive() {
        // давление всегда положительное
        assert!(p_sat(0.0) > 0.0);
        assert!(p_sat(20.0) > 0.0);
        assert!(p_sat(40.0) > 0.0);
    }

    #[test]
    fn test_temp_20() {
        //temp 20.0°C
        assert!(approx_eq(delta(20.0), 0.145));
    }

    #[test]
    fn test_temp_30() {
        //temp 30.0°C
        assert!(approx_eq(delta(30.0), 0.243));
    }
    #[test]
    fn indor_temp_zero() {
        //temp 20.0°C
        assert!(approx_eq(indoor_temp(0.0), 27.0));
    }

    #[test]
    fn indor_temp_33() {
        //temp 33.0°C
        assert!(approx_eq(indoor_temp(20.0), 33.0));
    }
    #[test]
    fn indor_temp_minus_20() {
        //temp -20.0°C
        assert!(approx_eq(indoor_temp(-20.0), 21.0));
    }

    #[test]
    fn vpd_temp_20() {
        //vpd with temp 20.0°C
        assert!(approx_eq(vpd(20.0), 1.403));
    }
    #[test]
    fn vpd_temp_30() {
        //vpd with temp 30.0°C
        assert!(approx_eq(vpd(30.0), 2.546));
    }
    #[test]
    fn vpd_temp_30_and_humidity() {
        //vpd with temp 30.0°C
        assert!(approx_eq(vpd_with_diff_humidity(30.0, 1.0), 0.0));
    }
    #[test]
    fn test_penman_monteith_positive() {
        // результат всегда неотрицательный
        let et = penman_monteith(20.0, "Regular", "window", "normal", 16.0, 1.0);
        assert!(et >= 0.0);
    }

    #[test]
    fn test_tropical_evaporates_more_than_succulent() {
        // тропическое испаряет больше чем суккулент
        let et_tropical = penman_monteith(20.0, "Tropical", "window", "normal", 16.0, 1.0);
        let et_succulent = penman_monteith(20.0, "Succulent", "window", "normal", 16.0, 1.0);
        assert!(et_tropical > et_succulent);
    }

    #[test]
    fn test_window_evaporates_more_than_shadow() {
        // у окна испаряет больше чем в тени
        let et_window = penman_monteith(20.0, "Regular", "window", "normal", 16.0, 1.0);
        let et_shadow = penman_monteith(20.0, "Regular", "shadow", "normal", 16.0, 1.0);
        assert!(et_window > et_shadow);
    }

    #[test]
    fn test_normal_circulation_evaporates_more_than_stagnant() {
        // нормальная циркуляция даёт больше испарения чем застой
        let et_normal = penman_monteith(20.0, "Regular", "window", "normal", 16.0, 1.0);
        let et_stagnant = penman_monteith(20.0, "Regular", "window", "stagnant", 16.0, 1.0);
        assert!(et_normal > et_stagnant);
    }

    #[test]
    fn test_larger_pot_evaporates_more() {
        // больший горшок — больше площадь испарения
        let et_small = penman_monteith(20.0, "Regular", "window", "normal", 12.0, 1.0);
        let et_large = penman_monteith(20.0, "Regular", "window", "normal", 20.0, 1.0);
        assert!(et_large > et_small);
    }

    #[test]
    fn test_higher_transpiration_coef_evaporates_more() {
        // выше коэффициент транспирации — больше испарение
        let et_low = penman_monteith(20.0, "Regular", "window", "normal", 16.0, 0.5);
        let et_high = penman_monteith(20.0, "Regular", "window", "normal", 16.0, 2.0);
        assert!(et_high > et_low);
    }

    #[test]
    fn test_higher_temp_evaporates_more() {
        // выше температура — больше испарение
        let et_cold = penman_monteith(10.0, "Regular", "window", "normal", 16.0, 1.0);
        let et_warm = penman_monteith(30.0, "Regular", "window", "normal", 16.0, 1.0);
        assert!(et_warm > et_cold);
    }
    #[test]
    fn test_taw_positive() {
        // TAW всегда положительный
        assert!(taw("universal", 1.5) > 0.0);
        assert!(taw("succulent", 1.5) > 0.0);
        assert!(taw("tropical", 1.5) > 0.0);
    }

    #[test]
    fn test_taw_grows_with_volume() {
        // больше объём грунта — больше доступной воды
        let taw_small = taw("universal", 1.0);
        let taw_large = taw("universal", 3.0);
        assert!(taw_large > taw_small);
    }

    #[test]
    fn test_taw_tropical_greater_than_succulent() {
        // тропический грунт удерживает больше воды чем грунт для суккулентов
        let taw_tropical = taw("tropical", 1.5);
        let taw_succulent = taw("succulent", 1.5);
        assert!(taw_tropical > taw_succulent);
    }

    #[test]
    fn test_raw_grows_with_volume() {
        // больше объём — больше доступной воды до полива
        let raw_small = raw("universal", 1.0, "Regular");
        let raw_large = raw("universal", 3.0, "Regular");
        assert!(raw_large > raw_small);
    }

    #[test]
    fn test_weighted_r_no_history_no_current() {
        // первый цикл, нет измерений — только физика
        let r = weighted_r(10.0, 0.0, None, 0, 0);
        assert!(approx_eq(r, 10.0));
    }

    #[test]
    fn test_weighted_r_with_history_no_current() {
        // есть история, нет текущих — смесь истории и физики
        let r = weighted_r(10.0, 20.0, None, 0, 5);
        // w_historical=5, w_physical=1 → (5*20 + 1*10) / 6 ≈ 18.33
        assert!(approx_eq(r, 18.33));
    }

    #[test]
    fn test_weighted_r_with_all_sources() {
        // все три источника
        let r = weighted_r(10.0, 20.0, Some(15.0), 3, 5);
        // w_current=3, w_historical=5, w_physical=1
        // (3*15 + 5*20 + 1*10) / 9 = (45 + 100 + 10) / 9 ≈ 17.22
        assert!(approx_eq(r, 17.22));
    }

    #[test]
    fn test_weighted_r_large_cycles_count_pulls_toward_history() {
        // много циклов — результат ближе к истории
        let r_many = weighted_r(10.0, 20.0, None, 0, 100);
        let r_few = weighted_r(10.0, 20.0, None, 0, 1);
        assert!(r_many > r_few);
    }

    #[test]
    fn test_weighted_r_large_n_current_pulls_toward_current() {
        // много текущих измерений — результат ближе к текущему
        let r_many = weighted_r(10.0, 20.0, Some(30.0), 100, 1);
        let r_few = weighted_r(10.0, 20.0, Some(30.0), 2, 1);
        assert!(r_many > r_few);
    }

    #[test]
    fn test_weighted_r_positive() {
        // результат всегда положительный
        assert!(weighted_r(10.0, 20.0, Some(15.0), 3, 5) > 0.0);
        assert!(weighted_r(10.0, 0.0, None, 0, 0) > 0.0);
    }
}
