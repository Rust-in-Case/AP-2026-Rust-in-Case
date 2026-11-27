use crate::map::{RicMap, RicMapErr};
use contractors::clues::Clue;
use contractors::conformance::chart::{
    Action, Chart, GenError, Outcome, RefChartTrait, Rejection, Report, ShipView, SpectatorView,
};
use contractors::geometry::{Cell, Map, Terrain};
use contractors::islandtrait::island::{Island, IslandInit};
use contractors::shared::{IslandId, Params, Round, Seed, ShipId};
use rand::{Rng, RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

/// A reference [`Chart`]
pub struct RicChart {
    map: RicMap,
    round: Round,
    _islands: Vec<Box<dyn Island>>,
    params: Params,
    _treasure: Cell,
    w_seed: Seed,
}

/// A reference [`Chart::ValidatedAction`]
#[derive(Debug)]
pub enum RefValidatedAction {}

impl Chart for RicChart {
    type ValidatedAction = RefValidatedAction;

    /// Generate the archipelago, returns a [`GenError`] if some step fails
    ///
    /// # Panics
    ///
    /// # Errors
    ///
    /// See [`GenError`] definitions
    fn generate(seed: Seed, params: &Params) -> Result<Self, GenError> {
        // 1. Seed reproducible random generator from the generation stream (not wall clock).
        let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(seed);

        let radius = params.archipelago_radius;

        // 2. Create sea hex grid of `archipelago_radius` . Reject if radius ≤ `vision_range`.
        if radius <= params.vision_range {
            return Err(GenError::VisionRangeTooHigh);
        }
        dbg!("Passed 2");

        let mut map = match Self::map_generate(radius) {
            Ok(val) => val,
            Err(error) => {
                println!("{error:?}");
                Self::empty_map()
            }
        };
        dbg!("Generated a map");

        // 3. Place `island_count` connected island footprints.
        let island_ids = Self::island_place(&mut rng, &mut map, params.island_count)?;
        dbg!("Placed islands");

        let retries = 3;

        // 4. Place `cliff_count` cliffs (must not touch islands or sea corners/adjacent cells)
        Self::cliff_place(&mut rng, &mut map, params.cliff_count, retries)?;
        dbg!("Placed cliffs");

        // 5. Place `rock_count` rocks on clear water (never on corners or adjacent cells).
        Self::rock_place(&mut rng, &mut map, params.rock_count, retries)?;
        dbg!("Placed rocks");

        // 6. Connectivity Check: Verify clear water is fully connected without crossing rocks
        if !Self::water_connected(
            &map,
            Cell {
                q: -radius.cast_signed(),
                r: 0,
            },
            &mut BTreeSet::new(),
        ) {
            return Err(GenError::WaterNotConnected);
        }
        dbg!("Water is connected");

        // 7. Treasure Placement: Select clear water cell not adjacent to land/corners. Retry if impossible.
        let treasure = Self::treasure_place(&mut rng, &map)?;
        dbg!("Treasure placed");

        // 8. Assign one CLUE to each Island (Section 4).
        let clues: Vec<Clue> = Self::clues_generate(&mut rng, treasure, params.island_count, &map, map.corners())?;
        dbg!("Clues generated");

        let islands = Self::clues_assign(&island_ids, &clues, &mut rng, params);
        dbg!("Clues assigned");

        // 9. Solvability Check: Verify full clue intersection equals exactly 1 cell. Advance seed and retry if failed
        Self::solvable(&map, &clues, treasure)?;
        dbg!("Solvable!");

        Ok(Self {
            map,
            round: 0,
            params: *params,
            _islands: islands,
            _treasure: treasure,
            w_seed: rng.next_u64(),
        })
    }

    /// Decide whether an action is legal. The only way to obtain a `ValidatedAction`.
    ///
    /// # Errors
    ///
    /// See [`Rejection`] definitions
    fn validate(&self, ship: ShipId, action: &Action) -> Result<Self::ValidatedAction, Rejection> {
        todo!()
    }

    /// Apply an already validated action. Cannot fail.
    fn apply(&mut self, action: Self::ValidatedAction) -> Vec<Report> {
        todo!()
    }

    /// End of round: the round number advances and the Islands take their turn.
    fn advance_round(&mut self) -> Vec<Report> {
        self.round += 1;
        todo!()
    }

    /// What a Ship is permitted to see.
    fn view_for(&self, ship: ShipId) -> Option<ShipView> {
        todo!()
    }

    /// What a spectator sees: everything.
    fn spectator_view(&self) -> SpectatorView {
        todo!()
    }

    /// Outcome of the game
    fn outcome(&self) -> Option<Outcome> {
        todo!()
    }

    /// Parameters passed at [`Chart::generate()`]
    fn params(&self) -> Params {
        self.params
    }

    /// The [`Chart`] [`Map`]
    fn map(&self) -> &impl Map {
        &self.map
    }

    /// The weather [`Seed`]
    fn weather_seed(&self) -> Seed {
        self.w_seed
    }

    /// The current [`Round`]
    fn round(&self) -> Round {
        self.round
    }
}

impl RefChartTrait for RicChart {}

impl RicChart {
    /// Assigns a [`Clue`] to an [`Island`]
    #[must_use]
    fn clues_assign<R: Rng>(
        island_ids: &[IslandId],
        clues: &[Clue],
        rng: &mut R,
        params: &Params,
    ) -> Vec<Box<dyn Island>> {
        let islands: Vec<Box<dyn Island>> = island_ids
            .iter()
            .zip(clues)
            .map(|(&id, &clue)| {
                let seed: Seed = rng.random();
                Self::island_create(id, clue, seed, *params)
            })
            .collect();

        islands
    }

    /// Empty [`RicMap`], used to recover invalid `map_generate`
    #[must_use]
    fn empty_map() -> RicMap {
        RicMap::empty()
    }

    /// Generates a new [`RicMap`]
    ///
    /// # Errors
    ///
    /// see [`RicMap::init`]
    fn map_generate(radius: u32) -> Result<RicMap, RicMapErr> {
        let cells: BTreeMap<Cell, Terrain> = Self::cells_generate(radius)?;
        let map = RicMap::init(radius, cells)?;

        Ok(map)
    }

    /// Generate a `BTreeMap<Cell,Terrain>` cells
    ///
    /// # Errors
    ///
    /// [`RicMap::InvalidRadius`] if the `radius` exceeds the bounds
    fn cells_generate(radius: u32) -> Result<BTreeMap<Cell, Terrain>, RicMapErr> {
        if !RicMap::radius_bound(radius) {
            return Err(RicMapErr::InvalidRadius);
        }

        let rad: i32 = radius.cast_signed();
        let neg_rad: i32 = -rad;

        let mut cells: BTreeMap<Cell, Terrain> = BTreeMap::new();

        for q in neg_rad..=rad {
            let r_min = neg_rad.max(neg_rad.saturating_sub(q));
            let r_max = rad.min(rad.saturating_sub(q));

            for r in r_min..=r_max {
                cells.insert(Cell { q, r }, Terrain::Clear);
            }
        }

        Ok(cells)
    }

    fn island_create(island_id: IslandId, clue: Clue, seed: Seed, params: Params) -> Box<dyn Island> {
        todo!()
    }
}
