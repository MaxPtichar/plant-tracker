use crate::models::MeasurementType;
/// FSM state for the "record measurement" dialogue.
///
/// Transitions:
/// ```text
/// WaitingForPlant
///     └─ (callback: plant selected) ──→ WaitingForWeight
///             └─ (text: float) ────────→ WaitingForType
///                     └─ (callback: type) ──→ WaitingForDate
///                             └─ (callback: date) ──→ [save → exit]
/// ```
///
/// Side dialogues entered from the main menu:
/// - `CreatingPlant` — add a new plant (sub-FSM)
/// - `CreatingPot` — configure pot for an existing plant (sub-FSM)
/// - `WaitingForPlantRecord` — view measurement history
/// - `WaitingForPlantDelete` / `WaitingForConfirmDelete` — delete a plant
#[derive(Debug, Clone, Default)]
pub enum MeasurementDialogue {
    /// Entry point. Waiting for the user to select a plant
    /// from the inline keyboard.
    #[default]
    WaitingForPlant,
    MyPlants,

    /// Waiting for the user to select a plant to view measurement history.
    WaitingForPlantRecord,

    /// Waiting for the user to select a plant to delete.
    WaitingForPlantDelete,

    /// Plant selected for deletion. Waiting for confirmation (`"ConfirmDelete"` or `"MyPlants"`).
    WaitingForConfirmDelete {
        plant_id: i64,
    },

    /// Pot configuration sub-FSM. See [`PotCreationDialog`].
    CreatingPot(PotCreationDialog),

    /// User chose to create a new plant instead of selecting existing.
    /// Delegates to [`PlantCreationDialogue`] sub-FSM.
    CreatingPlant(PlantCreationDialogue),

    /// Plant selected. Waiting for weight input (grams, float).
    WaitingForWeight {
        plant_id: i64,
        plant_name: String,
    },

    /// Weight collected. Waiting for measurement type selection
    /// via inline keyboard ([`MeasurementType`]).
    WaitingForType {
        plant_id: i64,
        weight: f32,
    },

    /// Type collected. Waiting for date selection via inline keyboard.
    WaitingForDate {
        plant_id: i64,
        weight: f32,
        type_: MeasurementType,
    },
}

/// FSM state for the "add new plant" dialogue.
///
/// Transitions:
/// ```text
/// WaitingForName
///     └─ (text: name) ──────────────→ WaitingForMoisture
///                 ├─ (callback: preset) ──→ [create plant → exit]
///                 └─ (callback: custom) ──→ WaitingForCustomMoisture
///                             └─ (text: float 0.0–1.0) ──→ [create plant → exit]
/// ```
#[derive(Debug, Clone, Default)]
pub enum PlantCreationDialogue {
    /// Entry point. Waiting for the plant's display name as a text message.
    #[default]
    WaitingForName,

    /// Name collected. Waiting for residual moisture threshold
    /// via inline keyboard — either a preset value or "custom".
    WaitingForMoisture { name: String },

    /// User chose custom moisture. Waiting for a float in range `0.0..=1.0`
    /// as a text message (commas normalised to dots).
    WaitingForCustomMoisture { name: String },
}

/// FSM state for the "configure pot" dialogue.
///
/// Transitions:
/// ```text
/// ChoosePlantName
///     └─ (callback: plant selected) ──→ WaitingForPotWeight
///             └─ (text: integer) ──────→ WaitingForDrySoilWeight
///                     └─ (text: integer) ──→ [save config → exit]
/// ```
#[derive(Debug, Clone, Default)]
pub enum PotCreationDialog {
    /// Entry point. Waiting for plant selection via inline keyboard.
    #[default]
    ChoosePlantName,

    /// Plant selected. Waiting for empty pot weight input (grams, integer).
    WaitingForPotWeight { plant_id: i64 },

    /// Pot weight collected. Waiting for dry soil weight input (grams, integer).
    WaitingForDrySoilWeight { plant_id: i64, pot_weight: i64 },
}
