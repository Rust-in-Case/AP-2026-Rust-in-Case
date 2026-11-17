use geometry::{Cell, Map, MapError, Terrain};
use rand::{
    seq::{IndexedRandom, IteratorRandom},
    Rng, RngExt, SeedableRng,
};
use rand_chacha::ChaCha20Rng;
use shared::{IslandId, Params, Seed, ShipId};
use std::collections::{BTreeMap, BTreeSet};

mod facade {
    use geometry::Cell;
    use shared::IslandId;
    use thiserror::Error;

    pub struct ShipView {}

    #[non_exhaustive]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum GenErrItem {
        Island,
        Cliff,
        Rock,
        Treasure,
    }

    #[non_exhaustive]
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Error)]
    pub enum GenError {
        // Step 2
        #[error("`archipelago_radius` <= `vision_range`.")]
        VisionRangeTooHigh,
        // Steps 3-5 and 7
        #[error("Cannot place `{item:?}`, the sea might be too crowded.")]
        CantPlace { item: GenErrItem },
        // Step 6
        #[error("The water is not connected by clear water alone.")]
        WaterNotConnected,
        // Step 9
        #[error("The Clues do not narrow the sea down to a single cell.")]
        Unsolvable,
    }

    pub enum Action {}

    pub struct ValidatedAction {}

    pub struct Rejection {}

    pub struct Event {}

    pub struct SpectatorView {}

    pub enum Outcome {}

    #[derive(Debug)]
    pub struct Island {
        pub id: IslandId,
    }

    #[non_exhaustive]
    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
    pub enum Clue {
        Within { centre: Cell, radius: u32 },
        Beyond { centre: Cell, radius: u32 },
        HalfPlane { axis: Axis, bound: Bound },
        Nearer { a: Cell, b: Cell },
    }
    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
    pub enum Axis {
        Q,
        R,
        S,
    }
    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
    pub enum Bound {
        AtLeast(i32),
        AtMost(i32),
    }
    impl Clue {
        pub fn holds(self, cell: Cell) -> bool {
            match self {
                Clue::Within { centre, radius } => cell.distance(centre) <= radius,
                Clue::Beyond { centre, radius } => cell.distance(centre) > radius,
                Clue::HalfPlane { axis, bound } => {
                    let v = match axis {
                        Axis::Q => cell.q,
                        Axis::R => cell.r,
                        Axis::S => cell.s(),
                    };
                    match bound {
                        Bound::AtLeast(k) => v >= k,
                        Bound::AtMost(k) => v <= k,
                    }
                }
                Clue::Nearer { a, b } => cell.distance(a) < cell.distance(b),
            }
        }
    }
}

use facade::{
    Action, Clue, Event, GenErrItem, GenError, Island, Outcome, Rejection, ShipView, SpectatorView, ValidatedAction,
};

#[derive(Debug)]
pub struct Chart {
    map: Map,
    islands: Vec<Island>,
    treasure: Cell,
}

// Temporary fix
impl From<MapError> for GenError {
    fn from(err: MapError) -> Self {
        match err {
            MapError::ErrRadius => GenError::VisionRangeTooHigh,
            MapError::ErrCellDup => GenError::VisionRangeTooHigh,
            MapError::ErrCellOut => GenError::VisionRangeTooHigh,
            MapError::ErrNumCell => GenError::VisionRangeTooHigh,
            _ => GenError::VisionRangeTooHigh,
        }
    }
}

/// Protocol - 8
impl Chart {
    /// Creates a new chart
    fn new(map: Map, treasure: Cell) -> Chart {
        Chart {
            map,
            islands: vec![],
            treasure,
        }
    }

    /// Generate a BTreeMap of <Cell,Terrain> cells
    fn cells_generate(radius: u32) -> BTreeMap<Cell, Terrain> {
        let rad: i64 = radius as i64;
        let neg_rad: i64 = -rad;

        let mut cells: BTreeMap<Cell, Terrain> = BTreeMap::new();

        for q in neg_rad..=rad {
            let r_min = neg_rad.max(neg_rad.saturating_sub(q));
            let r_max = rad.min(rad.saturating_sub(q));

            for r in r_min..=r_max {
                cells.insert(
                    Cell {
                        q: q as i32,
                        r: r as i32,
                    },
                    Terrain::Clear,
                );
            }
        }

        cells
    }

    /// Islands implementations
    fn islands_get<R>(rng: &mut R) -> Vec<Island>
    where
        R: Rng + Sized,
    {
        vec![
            Island {
                id: IslandId(rng.random::<u8>()),
            },
            Island {
                id: IslandId(rng.random::<u8>()),
            },
            Island {
                id: IslandId(rng.random::<u8>()),
            },
        ]
    }

    /// Used in step 6. of generate
    fn water_connected(map: &Map, start_cell: Cell, visited: &mut BTreeSet<Cell>) -> bool {
        if map.is_land(start_cell) {
            return false;
        }

        if !visited.insert(start_cell.clone()) {
            return true;
        }

        start_cell
            .ring(1)
            .filter(|c| {
                map.contains(c.clone())
                    && !map.is_land(c.clone())
                    && map.terrain(c.clone()).is_some_and(|t| t == Terrain::Rock)
            })
            .all(|c| Self::water_connected(map, c, visited))
    }

    /// Generates a clue
    fn clue_generate<R>(rng: &mut R, map: &Map, treasure: &Cell, max_dist: u32) -> Clue
    where
        R: Rng + Sized,
    {
        let variant = rng.random_range(0u8..=3);

        todo!()
    }

    fn _print_debug(map: &Map) {
        let radius = map.radius().cast_signed();

        for q in -radius..radius {
            for r in -radius..radius {
                let cell = Cell { q, r };

                if !map.contains(cell) {
                    print!("  ")
                } else {
                    match map.terrain(cell) {
                        Some(terr) => match terr {
                            Terrain::Clear => print!("W "),
                            Terrain::Rock => print!("R "),
                            Terrain::Cliff => print!("C "),
                            _ => print!(". "),
                        },
                        None => print!("? "),
                    }
                }
            }
            println!("");
        }
    }

    /// Main - 2.3 - How a sea is made
    pub fn generate(seed: Seed, params: &Params) -> Result<Chart, GenError> {
        // 1. Seed reproducible random generator from the generation stream (not wall clock).
        let mut rng: ChaCha20Rng = ChaCha20Rng::seed_from_u64(seed);

        // 2. Create sea hex grid of `archipelago_radius` . Reject if radius ≤ `vision_range` .
        if params.archipelago_radius <= params.vision_range {
            return Err(GenError::VisionRangeTooHigh);
        }

        let ap_radius = params.archipelago_radius;
        let mut cells = Self::cells_generate(ap_radius);
        let mut map = Map::try_new(ap_radius, cells.clone())?;

        let possible_islands = Self::islands_get(&mut rng);
        let num_islands = possible_islands.len() as u32;

        // TODO IMPLEMENT
        // 3. Place `island_count` connected island footprints.
        // Footprints must NOT touch each other, corners, or cells adjacent to corners.
        /* for i in 0..params.island_count {
            // let rand_island = (rng.random::<u32>() % num_islands);
            // println!("{rand_island}");
            todo!();
        } */

        let retries = 3;

        // 4. Place `cliff_count` cliffs (must not touch islands or sea corners/adjacent cells)
        let corners = map.corners();
        let mut excluded = BTreeSet::new();

        for c in corners {
            excluded.insert(c);
            excluded.extend(c.ring(1));
        }

        for _ in 0..params.cliff_count {
            let candidates: Vec<Cell> = map.clear_cells().filter(|c| !excluded.contains(c)).collect();

            let cliff_cell = (0..retries).find_map(|_| {
                let cliff_candidate = candidates.choose(&mut rng)?;

                cliff_candidate
                    .ring(1)
                    .filter(|c| map.contains(c.clone()))
                    .all(|c| !map.is_land(c))
                    .then_some(cliff_candidate)
            });

            let Some(cliff_selected) = cliff_cell else {
                return Err(GenError::CantPlace {
                    item: GenErrItem::Cliff,
                });
            };

            cells.insert(cliff_selected.clone(), Terrain::Cliff);
            map = Map::try_new(ap_radius, cells.clone())?;
        }

        // 5. Place `rock_count` rocks on clear water (never on corners or adjacent cells).
        let corners = map.corners();
        let mut excluded = BTreeSet::new();

        for cell in corners {
            excluded.insert(cell);
            excluded.extend(cell.ring(1));
        }

        for _ in 0..params.rock_count {
            let candidates: Vec<Cell> = map.clear_cells().filter(|c| !excluded.contains(c)).collect();

            let rock_cell = (0..retries).find_map(|_| {
                let rock_candidate = candidates.choose(&mut rng)?;

                Some(rock_candidate)
            });

            let Some(rock_selected) = rock_cell else {
                return Err(GenError::CantPlace { item: GenErrItem::Rock });
            };

            cells.insert(rock_selected.clone(), Terrain::Rock);
            map = Map::try_new(ap_radius, cells.clone())?;
        }

        // 6. Connectivity Check: Verify clear water is fully connected without crossing rocks.
        // Otherwise, advance seed and retry.
        if !Self::water_connected(
            &map,
            Cell {
                q: -ap_radius.cast_signed(),
                r: 0,
            },
            &mut BTreeSet::new(),
        ) {
            return Err(GenError::WaterNotConnected);
        }

        // 7. Treasure Placement: Select clear water cell not adjacent to land/corners. Retry if impossible.
        let corners = map.corners();
        let candidates: Vec<Cell> = map.clear_cells().filter(|c| !corners.contains(c)).collect();

        let treasure_cell = (0..1).find_map(|_| {
            let treasure_candidate = candidates.choose(&mut rng)?;

            treasure_candidate
                .ring(1)
                .filter(|c| map.contains(c.clone()))
                .all(|c| !map.is_land(c))
                .then_some(treasure_candidate)
        });

        let Some(treasure_selected) = treasure_cell else {
            return Err(GenError::CantPlace {
                item: GenErrItem::Treasure,
            });
        };

        // 8. Assign one CLUE to each Island (Section 4).
        todo!();

        let chart = Chart::new(map, treasure_selected.clone());

        // 9. Solvability Check: Verify full clue intersection equals exactly 1 cell. Advance seed and retry if failed
        todo!();

        Ok(chart)
    }

    /// Decide whether an action is legal. The only way to obtain a ValidatedAction.
    pub fn validate(&self, ship: ShipId, action: &Action) -> Result<ValidatedAction, Rejection> {
        todo!()
    }

    /// Apply an already validated action. Cannot fail.
    pub fn apply(&mut self, action: ValidatedAction) -> Vec<Event> {
        todo!()
    }

    /// End of round: the round number advances and the Islands take their turn.
    pub fn advance_round(&mut self) -> Vec<Event> {
        todo!()
    }

    /// What a Ship is permitted to see.
    pub fn view_for(&self, ship: ShipId) -> ShipView {
        todo!()
    }

    /// What a spectator sees: everything.
    pub fn spectator_view(&self) -> SpectatorView {
        todo!()
    }

    pub fn outcome(&self) -> Option<Outcome> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
