use contractors::geometry::{Cell, Map, Terrain};
use std::collections::BTreeMap;

pub type CellStorage = BTreeMap<Cell, Terrain>;

#[derive(Debug)]
pub struct RicMap {
    radius: u32,
    cells: CellStorage,
}

impl Map for RicMap {
    fn radius(&self) -> u32 {
        self.radius
    }

    fn contains(&self, cell: Cell) -> bool {
        if let Some(_) = self.cells.get(&cell) {
            true
        } else {
            false
        }
    }

    fn terrain(&self, cell: Cell) -> Option<Terrain> {
        if let Some(terr) = self.cells.get(&cell) {
            Some(terr.clone())
        } else {
            None
        }
    }

    fn is_land(&self, cell: Cell) -> bool {
        self.terrain(cell).is_some_and(|t| t.is_land())
    }
    fn is_water(&self, cell: Cell) -> bool {
        self.terrain(cell).is_some_and(|t| t.is_water())
    }
    /// The six corners, in direction order (counter-clockwise, East is the first). Ship i starts on corner i.
    fn corners(&self) -> [Cell; 6] {
        let rad = self.radius().cast_signed();
        [
            Cell { q: rad, r: 0 },
            Cell { q: rad, r: -rad },
            Cell { q: 0, r: -rad },
            Cell { q: -rad, r: 0 },
            Cell { q: -rad, r: rad },
            Cell { q: 0, r: rad },
        ]
    }
    /// Every cell in the sea, in a fixed order. Clear water and rocks.
    fn water_cells(&self) -> impl Iterator<Item = Cell> + '_ {
        Cell { q: 0, r: 0 }
            .disc(self.radius())
            .filter(|&cell| self.is_water(cell))
    }
    /// Water that is not a rock.
    fn clear_cells(&self) -> impl Iterator<Item = Cell> + '_ {
        Cell { q: 0, r: 0 }.disc(self.radius()).filter(|&cell| {
            if let Some(terr) = self.cells.get(&cell) {
                *terr == Terrain::Clear
            } else {
                false
            }
        })
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum MapErr {
    CellsNumber,
    CellsOutside,
}

impl RicMap {
    pub fn init(radius: u32, cells: CellStorage) -> Result<Self, MapErr> {
        //the formula for the expected len is: (3 * radius * (radius + 1) + 1) as usize;
        let expected_len: usize = radius
            .saturating_add(1)
            .saturating_mul(3)
            .saturating_mul(radius)
            .saturating_add(1) as usize;

        if cells.len() != expected_len {
            return Err(MapErr::CellsNumber);
        }

        let radius_int: i64 = radius as i64;

        let _ = cells
            .iter()
            .all(|(c, _)| {
                let q = c.q as i64;
                let r = c.r as i64;

                q <= radius_int && q >= -radius_int && r <= radius_int && r >= -radius_int
            })
            .ok_or(MapErr::CellsOutside);

        Ok(Self { radius, cells })
    }
}
