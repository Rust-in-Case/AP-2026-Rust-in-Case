use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use contractors::shared::Params;

/// Turn execution mode for an Archipelago match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TurnMode {
    /// Players take turns one after another.
    #[default]
    Sequential,
    /// All players submit and resolve turns together.
    Simultaneous,
}

impl TurnMode {
    /// Returns the uppercase button label for this turn mode.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Sequential => "SEQUENTIAL",
            Self::Simultaneous => "SIMULTANEOUS",
        }
    }

    /// Returns the opposite turn mode.
    #[must_use]
    pub const fn toggle(self) -> Self {
        match self {
            Self::Sequential => Self::Simultaneous,
            Self::Simultaneous => Self::Sequential,
        }
    }
}

impl fmt::Display for TurnMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Complete game configuration selected through the splash and setup screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameConfig {
    /// Random seed used to generate the archipelago chart.
    pub seed: u64,
    /// Selected turn execution mode.
    pub turn_mode: TurnMode,
    /// Customizable game parameters.
    pub params: Params,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            seed: 123,
            turn_mode: TurnMode::default(),
            params: Params::default(),
        }
    }
}

impl GameConfig {
    /// Creates a [`GameConfig`] initialized with a time-derived seed and default parameters.
    #[must_use]
    pub fn new_random() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(123, |d| u64::try_from(d.as_nanos()).unwrap_or(123));
        Self {
            seed,
            ..Self::default()
        }
    }

    /// Creates a [`GameConfig`] with the given `seed` and default mode and parameters.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed,
            ..Self::default()
        }
    }
}

/// Identifies each customizable field of [`Params`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    /// Radius of the archipelago in hex-grid steps.
    ArchipelagoRadius,
    /// Number of islands.
    IslandCount,
    /// Number of cliff cells.
    CliffCount,
    /// Number of rock cells.
    RockCount,
    /// Maximum ship vision range.
    VisionRange,
    /// Maximum ship bombard range.
    BombardRange,
    /// Damage dealt by bombardment.
    BombardDamage,
    /// Hull strength of a ship.
    HullStrength,
    /// Maximum number of rounds.
    RoundCap,
    /// Time limit per round in milliseconds.
    RoundDeadlineMs,
}

/// Ordered list of all customizable [`Params`] fields.
pub const PARAM_KINDS: [ParamKind; 10] = [
    ParamKind::ArchipelagoRadius,
    ParamKind::IslandCount,
    ParamKind::CliffCount,
    ParamKind::RockCount,
    ParamKind::VisionRange,
    ParamKind::BombardRange,
    ParamKind::BombardDamage,
    ParamKind::HullStrength,
    ParamKind::RoundCap,
    ParamKind::RoundDeadlineMs,
];

impl ParamKind {
    /// Returns the human-readable parameter label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ArchipelagoRadius => "Archipelago Radius",
            Self::IslandCount => "Island Count",
            Self::CliffCount => "Cliff Count",
            Self::RockCount => "Rock Count",
            Self::VisionRange => "Vision Range",
            Self::BombardRange => "Bombard Range",
            Self::BombardDamage => "Bombard Damage",
            Self::HullStrength => "Hull Strength",
            Self::RoundCap => "Round Cap",
            Self::RoundDeadlineMs => "Round Deadline (ms)",
        }
    }

    /// Returns the `(min, max, step)` bounds for interactive adjustment.
    #[must_use]
    pub const fn bounds(self) -> (u32, u32, u32) {
        match self {
            Self::IslandCount => (0, u8::MAX as u32, 1),
            Self::CliffCount | Self::RockCount => (0, u16::MAX as u32, 1),
            Self::ArchipelagoRadius
            | Self::VisionRange
            | Self::BombardRange
            | Self::BombardDamage
            | Self::HullStrength => (1, 100, 1),
            Self::RoundCap => (1, 10_000, 10),
            Self::RoundDeadlineMs => (10, 60_000, 50),
        }
    }

    /// Reads the current value of this parameter from `params`.
    #[must_use]
    pub const fn get(self, params: &Params) -> u32 {
        match self {
            Self::ArchipelagoRadius => params.archipelago_radius,
            Self::IslandCount => params.island_count as u32,
            Self::CliffCount => params.cliff_count as u32,
            Self::RockCount => params.rock_count as u32,
            Self::VisionRange => params.vision_range,
            Self::BombardRange => params.bombard_range,
            Self::BombardDamage => params.bombard_damage,
            Self::HullStrength => params.hull_strength,
            Self::RoundCap => params.round_cap,
            Self::RoundDeadlineMs => params.round_deadline_ms,
        }
    }

    /// Sets this parameter on `params`, clamping to [`Self::bounds`].
    pub fn set(self, params: &mut Params, value: u32) {
        let (min, max, _) = self.bounds();
        let clamped = value.clamp(min, max);
        match self {
            Self::ArchipelagoRadius => params.archipelago_radius = clamped,
            Self::IslandCount => {
                params.island_count = u8::try_from(clamped).unwrap_or(u8::MAX);
            }
            Self::CliffCount => {
                params.cliff_count = u16::try_from(clamped).unwrap_or(u16::MAX);
            }
            Self::RockCount => {
                params.rock_count = u16::try_from(clamped).unwrap_or(u16::MAX);
            }
            Self::VisionRange => params.vision_range = clamped,
            Self::BombardRange => params.bombard_range = clamped,
            Self::BombardDamage => params.bombard_damage = clamped,
            Self::HullStrength => params.hull_strength = clamped,
            Self::RoundCap => params.round_cap = clamped,
            Self::RoundDeadlineMs => params.round_deadline_ms = clamped,
        }
    }

    /// Increments or decrements this parameter by `steps * step_size`, clamping to [`Self::bounds`].
    pub fn adjust(self, params: &mut Params, steps: i32) {
        let current = self.get(params);
        let (_, _, step) = self.bounds();
        let delta = i64::from(steps) * i64::from(step);
        let next = (i64::from(current) + delta).max(0);
        let next_u32 = u32::try_from(next).unwrap_or(u32::MAX);
        self.set(params, next_u32);
    }
}
