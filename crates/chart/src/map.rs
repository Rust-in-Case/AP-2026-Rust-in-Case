use contractors::conformance::chart::MapUpdatableCells;
use contractors::geometry::{Cell, Map, Terrain};
use std::collections::BTreeMap;
use thiserror::Error;

/// A reference [`Map`], used in [`RefChart`]
#[derive(Debug, PartialEq)]
pub struct RicMap {
    radius: u32,
    cells: BTreeMap<Cell, Terrain>,
}

impl Map for RicMap {
    fn radius(&self) -> u32 {
        self.radius
    }

    fn contains(&self, cell: Cell) -> bool {
        self.cells.contains_key(&cell)
    }

    fn terrain(&self, cell: Cell) -> Option<Terrain> {
        self.cells.get(&cell).copied()
    }

    fn is_land(&self, cell: Cell) -> bool {
        self.terrain(cell).is_some_and(|t| t.is_land())
    }

    fn is_water(&self, cell: Cell) -> bool {
        self.terrain(cell).is_some_and(|t| t.is_water())
    }

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

    fn water_cells(&self) -> impl Iterator<Item = Cell> + '_ {
        Cell { q: 0, r: 0 }
            .disc(self.radius())
            .filter(|&cell| self.is_water(cell))
    }

    fn clear_cells(&self) -> impl Iterator<Item = Cell> + '_ {
        Cell { q: 0, r: 0 }.disc(self.radius()).filter(|cell| {
            if let Some(terr) = self.cells.get(cell) {
                *terr == Terrain::Clear
            } else {
                false
            }
        })
    }
}

impl RicMap {
    /// Checks if the radius is `<= (i32::MAX / 2)`
    pub fn radius_bound(radius: u32) -> bool {
        radius <= (i32::MAX / 2) as u32
    }

    /// An empty [`RicMap`], useful for recovering errors
    pub fn empty() -> Self {
        Self {
            radius: 0,
            cells: BTreeMap::new(),
        }
    }

    /// Creates a new [`RicMap`]
    ///
    /// # Errors
    ///
    /// Returns a [`RicMapErr::InvalidRadius`] if the `radius` is not valid.
    ///
    /// Returns a [`RicMapErr::CellsOutside`] If there are [`Cell`]s outside `radius`
    pub fn init(radius: u32, cells: BTreeMap<Cell, Terrain>) -> Result<Self, RicMapErr> {
        if !Self::radius_bound(radius) {
            return Err(RicMapErr::InvalidRadius);
        }

        let radius_int = i32::try_from(radius).map_err(|_| RicMapErr::InvalidRadius)?;

        let _ = cells
            .iter()
            .all(|(c, _)| c.q <= radius_int && c.q >= -radius_int && c.r <= radius_int && c.r >= -radius_int)
            .ok_or(RicMapErr::CellsOutside);

        Ok(Self { radius, cells })
    }
}

impl MapUpdatableCells for RicMap {
    type MapErr = RicMapErr;

    fn update_cell(&mut self, cell: Cell, terrain: Terrain) -> Result<bool, Self::MapErr> {
        if !self.contains(cell) {
            return Err(RicMapErr::CellsOutside);
        }

        self.cells.insert(cell, terrain);
        Ok(true)
    }
}

/// Error types for [`RicMap`]
#[derive(Error, Debug, PartialEq, Eq)]
pub enum RicMapErr {
    /// The radius exceeds `i32::MAX / 2`, or the expected cell count
    /// cannot be computed using `u32` arithmetic.
    #[error("invalid map radius")]
    InvalidRadius,
    /// Cells outside the map boundaries
    #[error("cell outside map boundaries")]
    CellsOutside,
}
