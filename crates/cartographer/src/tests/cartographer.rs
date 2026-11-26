use super::*;
use contractors::geometry::{Cell, Map, Terrain};
use crossterm::event::KeyCode;
use ratatui::layout::Rect;

struct MockMap {
    radius: u32,
}

impl Map for MockMap {
    fn radius(&self) -> u32 {
        self.radius
    }

    fn terrain(&self, cell: Cell) -> Option<Terrain> {
        let r_i = i32::try_from(self.radius).ok()?;
        if cell.q.abs() > r_i || cell.r.abs() > r_i || (cell.q + cell.r).abs() > r_i {
            return None;
        }
        match (cell.q, cell.r) {
            (0, 0) => Some(Terrain::Rock),
            (1, 0) => Some(Terrain::Cliff),
            _ => Some(Terrain::Clear),
        }
    }
}

#[test]
fn radius_one_renders_staggered_hex_rows() {
    let map = MockMap { radius: 1 };
    let app = App::new(&map);
    let viewport = Rect::new(0, 0, 12, 5);
    let lines = app.build_visible_map_lines(viewport);

    assert_eq!(lines.len(), 5);
    // Outside r = -1..=1 should be empty
    assert!(lines[0].spans.is_empty());
    assert!(lines[4].spans.is_empty());

    // r = -1: 2 cells (q = 0, 1)
    let row_top: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
    // r = 0: 3 cells (q = -1, 0, 1)
    let row_mid: String = lines[2].spans.iter().map(|s| s.content.as_ref()).collect();
    // r = 1: 2 cells (q = -1, 0)
    let row_bot: String = lines[3].spans.iter().map(|s| s.content.as_ref()).collect();

    assert_eq!(row_top, "     ~ ~ ");
    assert_eq!(row_mid, "    ~ 🪨⛰️");
    assert_eq!(row_bot, "     ~ ~ ");
}

#[test]
fn camera_panning_clamps_to_radius() {
    let map = MockMap { radius: 1 };
    let mut app = App::new(&map);

    app.handle_map_key(KeyCode::Right);
    app.handle_map_key(KeyCode::Right);
    assert_eq!(app.cam_q, 1);

    app.handle_map_key(KeyCode::Up);
    app.handle_map_key(KeyCode::Up);
    assert_eq!(app.cam_r, -1);

    app.handle_map_key(KeyCode::Char('c'));
    assert_eq!((app.cam_q, app.cam_r), (0, 0));
}
