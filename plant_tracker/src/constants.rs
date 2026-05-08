// Порог остаточной влаги (доля от 0.0 до 1.0)

pub const SUKKULENT: f32 = 0.15;
pub const TROPICAL: f32 = 0.40;
pub const REGULAR_PLANT: f32 = 0.30;

// ============================================================
// Physics constants for indoor plant evapotranspiration model
// Based on Penman-Monteith FAO-56, adapted for indoor conditions
// ============================================================

// --- Формула Антуана (August-Roche-Magnus approximation) ---

/// Base saturated vapour pressure at 0°C, kPa
/// Source: FAO-56, Allen et al. 1998
pub const ANTOINE_BASE: f32 = 0.6108;

/// Empirical coefficient from August-Roche-Magnus approximation
/// Derived from Clausius-Clapeyron equation, valid for 0–60°C
pub const ANTOINE_COEF: f32 = 17.27;

/// Temperature shape coefficient, °C
/// Shifts the curve to match measured vapour pressure data
pub const ANTOINE_TEMP: f32 = 237.3;

/// Differentiation constant from Antoine equation for delta calculation
/// Source: FAO-56, Allen et al. 1998
pub const DELTA_COEF: f32 = 4098.0;

// --- Физические константы ---

/// Latent heat of vaporization at 20°C, J/kg
pub const LAMBDA: f32 = 2_450_000.0;

/// Psychrometric constant at sea level, kPa/°C
pub const GAMMA: f32 = 0.067;

/// Air density at 20°C, kg/m³
pub const RHO_AIR: f32 = 1.2;

/// Specific heat capacity of air, J/(kg·K)
pub const CP_AIR: f32 = 1013.0;

// --- Комнатные условия ---

/// Typical indoor relative humidity in apartments, fraction 0.0..=1.0
pub const INDOOR_HUMIDITY: f32 = 0.40;

/// Net radiation balance at a sunny window, W/m²
pub const RN_WINDOW: f32 = 50.0;

/// Net radiation balance in shadow or north-facing window, W/m²
pub const RN_SHADOW: f32 = 15.0;

/// Aerodynamic resistance with normal air circulation, s/m
/// Based on indoor fytotron measurements (150–500 s/m range)
pub const RA_NORMAL: f32 = 250.0;

/// Aerodynamic resistance in stagnant air (corners, enclosed spaces), s/m
pub const RA_STAGNANT: f32 = 800.0;

/// Offset between outdoor API temperature and indoor temperature, °C
/// Indoor is typically 5–10°C warmer in winter, cooler in summer
pub const T_INDOOR_OFFSET: f32 = 7.0;

/// Baseline indoor temperature, °C
pub const T_INDOOR_BASE: f32 = 20.0;

/// Outdoor temperature influence factor (30% of outdoor affects indoor)
pub const T_OUTDOOR_FACTOR: f32 = 0.3;

// --- Сопротивление устьиц по типу растения, s/m ---

/// Stomatal resistance for tropical plants.
/// Stomata fully open while water is available.
pub const RC_TROPICAL: f32 = 100.0;

/// Stomatal resistance for regular houseplants.
/// Similar to FAO-56 reference crop.
pub const RC_REGULAR: f32 = 300.0;

/// Stomatal resistance for succulents (CAM photosynthesis), averaged over 24h.
/// Day (~14h): stomata nearly closed (~10 000 s/m)
/// Night (~10h): stomata open (~200 s/m)
/// rc = (10_000 * 14 + 200 * 10) / 24
pub const RC_SUCCULENT: f32 = 5_916.7;

// --- Порог полива (Management Allowed Depletion) ---

/// Fraction of TAW that can be depleted before watering for tropical plants.
/// Water-sensitive — wilt quickly under drought stress.
pub const MAD_TROPICAL: f32 = 0.30;

/// Fraction of TAW that can be depleted before watering for regular houseplants.
pub const MAD_REGULAR: f32 = 0.65;

/// Fraction of TAW that can be depleted before watering for succulents.
/// Drought-tolerant — can lose up to 90% of available water.
pub const MAD_SUCCULENT: f32 = 0.90;

// --- Параметры грунта (Field Capacity / Permanent Wilting Point), g/l ---

/// Field Capacity for universal potting mix, g of water per liter of soil
pub const FC_UNIVERSAL: f32 = 600.0;
/// Permanent Wilting Point for universal potting mix, g/l
pub const PWP_UNIVERSAL: f32 = 80.0;

/// Field Capacity for succulent/cactus mix (coarse, fast-draining), g/l
pub const FC_SUCCULENT: f32 = 350.0;
/// Permanent Wilting Point for succulent/cactus mix, g/l
pub const PWP_SUCCULENT: f32 = 30.0;

/// Field Capacity for tropical mix (peat-based, moisture-retaining), g/l
pub const FC_TROPICAL: f32 = 400.0;
/// Permanent Wilting Point for tropical mix, g/l
pub const PWP_TROPICAL: f32 = 150.0;

// плотность кг/л
pub const BULK_DENSITY_UNIVERSAL: f32 = 0.5;
pub const BULK_DENSITY_SUCCULENT: f32 = 0.7;
pub const BULK_DENSITY_TROPICAL: f32 = 0.4;
