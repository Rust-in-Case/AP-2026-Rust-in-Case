use geometry::{Cell, Map, MapError, Terrain};
use rand::{seq::IndexedRandom, RngExt, SeedableRng};
use rand_chacha::ChaCha20Rng;
use shared::{IslandId, Params, Seed, ShipId};
use std::collections::{BTreeMap, BTreeSet};

mod facade {
    use shared::IslandId;

    pub struct ShipView {}

    pub enum GenError {
        Stub,
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
}

use facade::{Action, Event, GenError, Island, Outcome, Rejection, ShipView, SpectatorView, ValidatedAction};

#[derive(Debug)]
pub struct Chart {
    map: Map,
    islands: Vec<Island>,
    treasure: Cell,
}

impl From<MapError> for GenError {
    fn from(err: MapError) -> Self {
        match err {
            MapError::ErrRadius => GenError::Stub,
            MapError::ErrCellDup => GenError::Stub,
            MapError::ErrCellOut => GenError::Stub,
            MapError::ErrNumCell => GenError::Stub,
            _ => GenError::Stub,
        }
    }
}

/// Protocol - 8
impl Chart {
    fn new(map: Map, treasure: Cell) -> Chart {
        Chart {
            map,
            islands: vec![],
            treasure,
        }
    }

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

    fn islands_get(rng: &mut ChaCha20Rng) -> Vec<Island> {
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

    fn water_connected(map: &Map, start_cell: Cell) -> bool {
        let mut visited: BTreeSet<Cell> = BTreeSet::new();
        Self::water_connected_inn(map, start_cell, &mut visited)
    }

    fn water_connected_inn(map: &Map, start_cell: Cell, visited: &mut BTreeSet<Cell>) -> bool {
        if map.is_land(start_cell) {
            return false;
        }

        if !visited.insert(start_cell.clone()) {
            return true;
        }

        start_cell
            .ring(1)
            .filter(|c| map.contains(c.clone()) && !map.is_land(c.clone()))
            .all(|c| Self::water_connected_inn(map, c, visited))
    }

    /// Main - 2.3 - How a sea is made
    pub fn generate(seed: Seed, params: &Params) -> Result<Chart, MapError> {
        // 1. Seed reproducible random generator from the generation stream (not wall clock).
        let mut rng: ChaCha20Rng = ChaCha20Rng::seed_from_u64(seed);

        // TODO UPDATE ERR
        // 2. Create sea hex grid of `archipelago_radius` . Reject if radius ≤ `vision_range` .
        if params.archipelago_radius <= params.vision_range {
            return Err(MapError::ErrRadius);
        }

        let ap_radius = params.archipelago_radius;
        let mut cells = Self::cells_generate(ap_radius);
        let mut map = Map::try_new(ap_radius, cells.clone())?;

        let possible_islands = Chart::islands_get(&mut rng);
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
        for _ in 0..params.cliff_count {
            let corners = map.corners();
            let candidates: Vec<Cell> = map.clear_cells().filter(|c| !corners.contains(c)).collect();

            let found = (0..retries).find_map(|_| {
                let cand = candidates.choose(&mut rng)?;

                cand.ring(1)
                    .filter(|c| map.contains(c.clone()))
                    .all(|c| !map.is_land(c))
                    .then_some(cand)
            });

            // TODO update err
            let Some(cand) = found else {
                return Err(MapError::ErrRadius);
            };

            cells.insert(cand.clone(), Terrain::Cliff);
            map = Map::try_new(ap_radius, cells.clone())?;
        }

        // 5. Place `rock_count` rocks on clear water (never on corners or adjacent cells).
        for _ in 0..params.rock_count {
            let corners = map.corners();
            let candidates: Vec<Cell> = map.clear_cells().filter(|c| !corners.contains(c)).collect();

            let found = (0..retries).find_map(|_| {
                let cand = candidates.choose(&mut rng)?;

                cand.ring(1)
                    .filter(|c| map.contains(c.clone()))
                    .all(|c| !map.is_land(c))
                    .then_some(cand)
            });

            // TODO update err
            let Some(cand) = found else {
                return Err(MapError::ErrRadius);
            };

            cells.insert(cand.clone(), Terrain::Rock);
            map = Map::try_new(ap_radius, cells.clone())?;
        }

        // 6. Connectivity Check: Verify clear water is fully connected without crossing rocks.
        // Otherwise, advance seed and retry.
        // TODO: Add errors and fix
        Self::water_connected(
            &map,
            Cell {
                q: -ap_radius.cast_signed(),
                r: 0,
            },
        );

        // 7. Treasure Placement: Select clear water cell not adjacent to land/corners. Retry if impossible.
        let corners = map.corners();
        let candidates: Vec<Cell> = map.clear_cells().filter(|c| !corners.contains(c)).collect();

        let treasure_cand = (0..1).find_map(|_| {
            let cand = candidates.choose(&mut rng)?;

            cand.ring(1)
                .filter(|c| map.contains(c.clone()))
                .all(|c| !map.is_land(c))
                .then_some(cand)
        });

        // TODO update err
        let Some(treasure_cell) = treasure_cand else {
            return Err(MapError::ErrRadius);
        };

        // 8. Assign one CLUE to each Island (Section 4).
        todo!();

        let chart = Chart::new(map, treasure_cell.clone());

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
