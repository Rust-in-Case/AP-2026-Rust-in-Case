use std::io;

use contractors::geometry::{Cell, Map, Terrain};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::config::GameConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Input,
    Map,
}

/// Interactive TUI application state for exploring a [`Map`] alongside an input box.
pub struct App<'a, M: Map> {
    map: &'a M,
    /// Optional game configuration selected in the splash/setup screens.
    config: Option<GameConfig>,
    /// Single textbox buffer in the left panel
    input: String,
    /// Cursor byte/char index inside `input`
    cursor_pos: usize,
    /// Camera center in axial coordinates (q, r)
    cam_q: i32,
    cam_r: i32,
    /// Which pane currently receives keyboard input
    focus: Focus,
    should_quit: bool,
}

impl<'a, M: Map> App<'a, M> {
    /// Creates a new [`App`] centered at `(0, 0)` on the provided `map` with map focus active.
    #[must_use]
    pub fn new(map: &'a M) -> Self {
        Self {
            map,
            config: None,
            input: String::new(),
            cursor_pos: 0,
            cam_q: 0,
            cam_r: 0,
            focus: Focus::Map,
            should_quit: false,
        }
    }

    /// Creates a new [`App`] with the selected [`GameConfig`] displayed in the side panel.
    #[must_use]
    pub fn with_config(map: &'a M, config: GameConfig) -> Self {
        Self {
            config: Some(config),
            ..Self::new(map)
        }
    }

    /// Runs the event/render loop on `terminal` until the user quits with `Esc`.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if drawing to `terminal` or reading terminal events fails.
    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let Event::Key(key) = event::read()? else {
            return Ok(());
        };
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        // Global shortcuts
        match key.code {
            KeyCode::Esc => {
                self.should_quit = true;
                return Ok(());
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Input => Focus::Map,
                    Focus::Map => Focus::Input,
                };
                return Ok(());
            }
            _ => {}
        }

        match self.focus {
            Focus::Input => self.handle_input_key(key.code),
            Focus::Map => self.handle_map_key(key.code),
        }

        Ok(())
    }

    fn handle_input_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_pos, c);
                self.cursor_pos += c.len_utf8();
            }
            KeyCode::Backspace if self.cursor_pos > 0 => {
                let prev = self.input[..self.cursor_pos]
                    .char_indices()
                    .last()
                    .map_or(0, |(idx, _)| idx);
                self.input.remove(prev);
                self.cursor_pos = prev;
            }
            KeyCode::Left if self.cursor_pos > 0 => {
                self.cursor_pos = self.input[..self.cursor_pos]
                    .char_indices()
                    .last()
                    .map_or(0, |(idx, _)| idx);
            }
            KeyCode::Right if self.cursor_pos < self.input.len() => {
                let next = self.input[self.cursor_pos..]
                    .char_indices()
                    .nth(1)
                    .map_or(self.input.len(), |(idx, _)| self.cursor_pos + idx);
                self.cursor_pos = next;
            }
            KeyCode::Enter => {
                self.focus = Focus::Map;
            }
            _ => {}
        }
    }

    fn handle_map_key(&mut self, code: KeyCode) {
        let radius = i32::try_from(self.map.radius()).unwrap_or(i32::MAX);
        match code {
            KeyCode::Up | KeyCode::Char('w') => self.cam_r -= 1,
            KeyCode::Down | KeyCode::Char('s') => self.cam_r += 1,
            KeyCode::Left | KeyCode::Char('a') => self.cam_q -= 1,
            KeyCode::Right | KeyCode::Char('d') => self.cam_q += 1,
            KeyCode::Char('c') => {
                self.cam_q = 0;
                self.cam_r = 0;
            }
            _ => {}
        }

        self.cam_q = self.cam_q.clamp(-radius, radius);
        self.cam_r = self.cam_r.clamp(-radius, radius);
    }

    fn draw(&self, frame: &mut Frame) {
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Length(34), Constraint::Min(10)]).areas(frame.area());

        self.draw_left_panel(frame, left_area);
        self.draw_map_panel(frame, right_area);
    }

    fn draw_left_panel(&self, frame: &mut Frame, area: Rect) {
        let [textbox_area, info_area] = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);

        let is_focused = self.focus == Focus::Input;
        let border_style = if is_focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let title = if is_focused {
            "Command / Input (Tab: Map)"
        } else {
            "Command / Input (Tab: Edit)"
        };

        let input_widget = Paragraph::new(self.input.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        );

        frame.render_widget(input_widget, textbox_area);

        if let Some(cfg) = self.config {
            let p = &cfg.params;
            let lines = vec![
                Line::from(format!("Mode: {}", cfg.turn_mode)),
                Line::from(format!("Seed: {}", cfg.seed)),
                Line::from(format!("Radius: {}", p.archipelago_radius)),
                Line::from(format!("Islands: {}", p.island_count)),
                Line::from(format!("Cliffs: {}  Rocks: {}", p.cliff_count, p.rock_count)),
                Line::from(format!("Vision: {}  Bombard: {}", p.vision_range, p.bombard_range)),
                Line::from(format!("Dmg: {}  Hull: {}", p.bombard_damage, p.hull_strength)),
                Line::from(format!("Rounds: {}  Deadline: {}ms", p.round_cap, p.round_deadline_ms)),
            ];
            let info_widget = Paragraph::new(lines).style(Style::default().fg(Color::Gray)).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Match Config")
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            frame.render_widget(info_widget, info_area);
        }

        if is_focused {
            let char_offset = u16::try_from(self.input[..self.cursor_pos].chars().count()).unwrap_or(u16::MAX);
            let cursor_x = textbox_area
                .x
                .saturating_add(1)
                .saturating_add(char_offset)
                .min(textbox_area.right().saturating_sub(2));
            let cursor_y = textbox_area.y + 1;
            frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        }
    }

    fn draw_map_panel(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == Focus::Map;
        let border_style = if is_focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                "Map [q: {}, r: {}] (WASD/Arrows: Move | Tab: Input | Esc: Quit)",
                self.cam_q, self.cam_r
            ))
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let lines = self.build_visible_map_lines(inner);
        frame.render_widget(Paragraph::new(lines), inner);
    }

    fn build_visible_map_lines(&self, viewport: Rect) -> Vec<Line<'static>> {
        let radius = i32::try_from(self.map.radius()).unwrap_or(i32::MAX);
        let half_h = i32::from(viewport.height) / 2;
        let half_w_cells = i32::from(viewport.width) / 4;

        let mut lines = Vec::with_capacity(viewport.height as usize);

        for screen_y in 0..i32::from(viewport.height) {
            let r = self.cam_r + (screen_y - half_h);

            if r < -radius || r > radius {
                lines.push(Line::default());
                continue;
            }

            let q_min = (-radius).max(-r - radius);
            let q_max = radius.min(radius - r);

            let center_screen_x = i32::from(viewport.width) / 2;
            let cam_offset_x = self.cam_q * 2 + self.cam_r;

            let first_q = (self.cam_q - half_w_cells - 2).max(q_min);
            let last_q = (self.cam_q + half_w_cells + 2).min(q_max);

            let mut spans: Vec<Span<'static>> = Vec::new();
            let mut cursor_col = 0i32;

            for q in first_q..=last_q {
                let target_col = center_screen_x + (q * 2 + r) - cam_offset_x;

                if target_col < 0 || target_col + 2 > i32::from(viewport.width) {
                    continue;
                }

                if let Ok(pad) = usize::try_from(target_col - cursor_col)
                    && pad > 0
                {
                    spans.push(Span::raw(" ".repeat(pad)));
                    cursor_col = target_col;
                }

                let is_center = q == self.cam_q && r == self.cam_r;
                spans.push(terrain_span(self.map.terrain(Cell { q, r }), is_center));
                cursor_col += 2;
            }

            lines.push(Line::from(spans));
        }

        lines
    }
}

/// Renders every tile into a consistent 2-column cell so emojis and ASCII align cleanly.
fn terrain_span(t: Option<Terrain>, highlighted: bool) -> Span<'static> {
    let (symbol, mut style) = match t {
        Some(Terrain::Clear) => ("~ ", Style::default().fg(Color::Cyan)),
        Some(Terrain::Cliff) => ("⛰️", Style::default().fg(Color::Gray)),
        Some(Terrain::Island(_)) => ("🏝️", Style::default().fg(Color::Green)),
        Some(Terrain::Rock) => ("🪨", Style::default().fg(Color::DarkGray)),
        None => ("_ ", Style::default().fg(Color::DarkGray)),
    };

    if highlighted {
        style = style.add_modifier(Modifier::SLOW_BLINK);
    }

    Span::styled(symbol, style)
}

/// Launches the interactive Ratatui TUI to render and pan over `map`.
///
/// Displays a minimal two-pane layout with a single input box on the left and
/// the pannable hexagonal `map` on the right, returning once the user exits
/// with `Esc`.
///
/// # Errors
///
/// Returns an [`io::Error`] if terminal initialization, rendering, or event
/// handling fails.
pub fn render(map: &impl Map) -> io::Result<()> {
    ratatui::run(|terminal| App::new(map).run(terminal))
}

#[cfg(test)]
#[path = "tests/cartographer.rs"]
mod tests;
