use std::io;

use contractors::shared::Params;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::config::{GameConfig, PARAM_KINDS, ParamKind, TurnMode};
use crate::title_art;

const ACCENT_YELLOW: Color = Color::Rgb(245, 220, 110);
const ACTIVE_FG: Color = Color::Rgb(245, 250, 225);
const TEAL_ACCENT: Color = Color::Rgb(55, 165, 145);
const INACTIVE_FG: Color = Color::Gray;
const MUTED_FG: Color = Color::DarkGray;

/// Result of running the splash and setup flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplashOutcome {
    /// User completed configuration and chose to start the game.
    Start(GameConfig),
    /// User chose to quit the application.
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainMenuButton {
    NewGame,
    FromSeed,
    Quit,
}

impl MainMenuButton {
    const ALL: [Self; 3] = [Self::NewGame, Self::FromSeed, Self::Quit];

    const fn label(self) -> &'static str {
        match self {
            Self::NewGame => "NEW GAME",
            Self::FromSeed => "FROM SEED",
            Self::Quit => "QUIT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SetupFocus {
    Mode,
    ParametersHeader,
    Param(usize),
    Start,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    MainMenu,
    SeedInput,
    GameSetup,
}

/// Interactive state machine for the splash, seed entry, and game setup screens.
pub struct SplashScreen {
    screen: Screen,
    main_idx: usize,
    seed_input: String,
    seed_cursor: usize,
    seed_error: Option<&'static str>,
    config: GameConfig,
    setup_focus: SetupFocus,
    params_expanded: bool,
    param_edit_buf: Option<(usize, String)>,
    outcome: Option<SplashOutcome>,
}

impl Default for SplashScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl SplashScreen {
    /// Creates a new [`SplashScreen`] starting on the main menu.
    #[must_use]
    pub fn new() -> Self {
        Self {
            screen: Screen::MainMenu,
            main_idx: 0,
            seed_input: String::new(),
            seed_cursor: 0,
            seed_error: None,
            config: GameConfig::default(),
            setup_focus: SetupFocus::Mode,
            params_expanded: true,
            param_edit_buf: None,
            outcome: None,
        }
    }

    /// Returns the terminal outcome once the user has started a game or quit.
    #[must_use]
    pub const fn outcome(&self) -> Option<SplashOutcome> {
        self.outcome
    }

    /// Runs the splash screen loop until the user starts a game or quits.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if drawing to `terminal` or reading events fails.
    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<SplashOutcome> {
        while self.outcome.is_none() {
            terminal.draw(|frame| self.draw(frame))?;
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            self.handle_key(key.code);
        }
        Ok(self.outcome.unwrap_or(SplashOutcome::Quit))
    }

    /// Processes a single key press on the active screen.
    pub fn handle_key(&mut self, code: KeyCode) {
        match self.screen {
            Screen::MainMenu => self.handle_main_menu_key(code),
            Screen::SeedInput => self.handle_seed_input_key(code),
            Screen::GameSetup => self.handle_game_setup_key(code),
        }
    }

    fn handle_main_menu_key(&mut self, code: KeyCode) {
        let count = MainMenuButton::ALL.len();
        match code {
            KeyCode::Up | KeyCode::Char('w' | 'k') => {
                self.main_idx = (self.main_idx + count - 1) % count;
            }
            KeyCode::Down | KeyCode::Char('s' | 'j') => {
                self.main_idx = (self.main_idx + 1) % count;
            }
            KeyCode::Enter | KeyCode::Char(' ') => match MainMenuButton::ALL[self.main_idx] {
                MainMenuButton::NewGame => {
                    self.config = GameConfig::new_random();
                    self.setup_focus = SetupFocus::Mode;
                    self.param_edit_buf = None;
                    self.screen = Screen::GameSetup;
                }
                MainMenuButton::FromSeed => {
                    self.seed_input.clear();
                    self.seed_cursor = 0;
                    self.seed_error = None;
                    self.screen = Screen::SeedInput;
                }
                MainMenuButton::Quit => {
                    self.outcome = Some(SplashOutcome::Quit);
                }
            },
            KeyCode::Esc | KeyCode::Char('q') => {
                self.outcome = Some(SplashOutcome::Quit);
            }
            _ => {}
        }
    }

    fn handle_seed_input_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.seed_error = None;
                self.screen = Screen::MainMenu;
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.seed_input.insert(self.seed_cursor, c);
                self.seed_cursor += 1;
                self.seed_error = None;
            }
            KeyCode::Backspace if self.seed_cursor > 0 => {
                self.seed_cursor -= 1;
                self.seed_input.remove(self.seed_cursor);
                self.seed_error = None;
            }
            KeyCode::Left if self.seed_cursor > 0 => {
                self.seed_cursor -= 1;
            }
            KeyCode::Right if self.seed_cursor < self.seed_input.len() => {
                self.seed_cursor += 1;
            }
            KeyCode::Enter => {
                if self.seed_input.is_empty() {
                    self.seed_error = Some("Please enter a numeric seed.");
                    return;
                }
                if let Ok(seed) = self.seed_input.parse::<u64>() {
                    self.config = GameConfig::with_seed(seed);
                    self.setup_focus = SetupFocus::Mode;
                    self.param_edit_buf = None;
                    self.screen = Screen::GameSetup;
                } else {
                    self.seed_error = Some("Seed must fit in an unsigned 64-bit integer.");
                }
            }
            _ => {}
        }
    }

    fn handle_game_setup_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.commit_param_edit();
                self.screen = Screen::MainMenu;
                return;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.commit_param_edit();
                self.move_setup_focus(-1);
                return;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.commit_param_edit();
                self.move_setup_focus(1);
                return;
            }
            _ => {}
        }

        match self.setup_focus {
            SetupFocus::Mode => self.handle_mode_key(code),
            SetupFocus::ParametersHeader => self.handle_params_header_key(code),
            SetupFocus::Param(idx) => self.handle_param_row_key(idx, code),
            SetupFocus::Start => {
                if matches!(code, KeyCode::Enter | KeyCode::Char(' ')) {
                    self.outcome = Some(SplashOutcome::Start(self.config));
                }
            }
            SetupFocus::Back => {
                if matches!(code, KeyCode::Enter | KeyCode::Char(' ')) {
                    self.screen = Screen::MainMenu;
                }
            }
        }
    }

    fn handle_mode_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Left => self.config.turn_mode = TurnMode::Sequential,
            KeyCode::Right => self.config.turn_mode = TurnMode::Simultaneous,
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.config.turn_mode = self.config.turn_mode.toggle();
            }
            _ => {}
        }
    }

    fn handle_params_header_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.params_expanded = !self.params_expanded;
            }
            KeyCode::Char('r') => {
                self.config.params = Params::default();
            }
            _ => {}
        }
    }

    fn handle_param_row_key(&mut self, idx: usize, code: KeyCode) {
        let kind = PARAM_KINDS[idx];
        match code {
            KeyCode::Left | KeyCode::Char('-') => {
                self.commit_param_edit();
                kind.adjust(&mut self.config.params, -1);
            }
            KeyCode::Right | KeyCode::Char('+' | '=') => {
                self.commit_param_edit();
                kind.adjust(&mut self.config.params, 1);
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let buf = match &mut self.param_edit_buf {
                    Some((active_idx, s)) if *active_idx == idx => s,
                    slot => {
                        *slot = Some((idx, String::new()));
                        &mut slot.as_mut().expect("just assigned").1
                    }
                };
                if buf.len() < 10 {
                    buf.push(c);
                }
            }
            KeyCode::Backspace => {
                if let Some((active_idx, s)) = &mut self.param_edit_buf
                    && *active_idx == idx
                {
                    s.pop();
                } else {
                    let mut s = kind.get(&self.config.params).to_string();
                    s.pop();
                    self.param_edit_buf = Some((idx, s));
                }
            }
            KeyCode::Enter => {
                self.commit_param_edit();
            }
            KeyCode::Char('r') => {
                self.param_edit_buf = None;
                let default_val = kind.get(&Params::default());
                kind.set(&mut self.config.params, default_val);
            }
            _ => {}
        }
    }

    fn commit_param_edit(&mut self) {
        if let Some((idx, buf)) = self.param_edit_buf.take()
            && let Ok(val) = buf.parse::<u32>()
        {
            PARAM_KINDS[idx].set(&mut self.config.params, val);
        }
    }

    fn move_setup_focus(&mut self, dir: i32) {
        let mut items = vec![SetupFocus::Mode, SetupFocus::ParametersHeader];
        if self.params_expanded {
            for idx in 0..PARAM_KINDS.len() {
                items.push(SetupFocus::Param(idx));
            }
        }
        items.push(SetupFocus::Start);
        items.push(SetupFocus::Back);

        let pos = items.iter().position(|&item| item == self.setup_focus).unwrap_or(0);
        let len = items.len();
        let next = if dir < 0 {
            (pos + len - 1) % len
        } else {
            (pos + 1) % len
        };
        self.setup_focus = items[next];
    }

    /// Renders the active splash or setup screen.
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Block::default().style(Style::default().bg(Color::Black)), area);

        match self.screen {
            Screen::MainMenu => self.draw_main_menu(frame, area),
            Screen::SeedInput => self.draw_seed_input(frame, area),
            Screen::GameSetup => self.draw_game_setup(frame, area),
        }
    }

    fn draw_main_menu(&self, frame: &mut Frame, area: Rect) {
        let title_h = title_art::preferred_height(area.width);
        let [_, title_area, _, menu_area, footer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(title_h),
            Constraint::Length(2),
            Constraint::Length(11),
            Constraint::Length(2),
        ])
        .areas(area);

        title_art::render_title(frame, title_area);

        let [b0, b1, b2] =
            Layout::vertical([Constraint::Length(3), Constraint::Length(3), Constraint::Length(3)]).areas(menu_area);

        let slots = [b0, b1, b2];
        for (idx, button) in MainMenuButton::ALL.iter().enumerate() {
            render_menu_button(frame, slots[idx], button.label(), idx == self.main_idx);
        }

        let hint = Paragraph::new(Line::from(Span::styled(
            "Up/Down: Select   Enter: Confirm   Esc: Quit",
            Style::default().fg(MUTED_FG),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(hint, footer_area);
    }

    fn draw_seed_input(&self, frame: &mut Frame, area: Rect) {
        let [_, title_area, _, box_area, err_area, _, footer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        let header = Paragraph::new(Line::from(Span::styled(
            "FROM SEED",
            Style::default().fg(ACTIVE_FG).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(header, title_area);

        let centered = centered_rect(box_area, 38);
        let input = Paragraph::new(self.seed_input.as_str())
            .style(Style::default().fg(ACTIVE_FG))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Enter Numeric Seed ")
                    .border_style(Style::default().fg(ACTIVE_FG)),
            );
        frame.render_widget(input, centered);

        let cursor_x = centered
            .x
            .saturating_add(1)
            .saturating_add(u16::try_from(self.seed_cursor).unwrap_or(u16::MAX))
            .min(centered.right().saturating_sub(2));
        frame.set_cursor_position(Position::new(cursor_x, centered.y + 1));

        if let Some(err) = self.seed_error {
            let msg = Paragraph::new(Line::from(Span::styled(err, Style::default().fg(Color::Red))))
                .alignment(Alignment::Center);
            frame.render_widget(msg, err_area);
        }

        let hint = Paragraph::new(Line::from(Span::styled(
            "0-9: Type Seed   Enter: Continue   Esc: Back",
            Style::default().fg(MUTED_FG),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(hint, footer_area);
    }

    fn draw_game_setup(&self, frame: &mut Frame, area: Rect) {
        let param_rows = if self.params_expanded {
            u16::try_from(PARAM_KINDS.len()).unwrap_or(10) + 2
        } else {
            0
        };

        let [
            header_area,
            mode_area,
            params_btn_area,
            params_list_area,
            actions_area,
            footer_area,
        ] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(param_rows),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .areas(area);

        let header = Paragraph::new(vec![
            Line::from(Span::styled(
                "NEW GAME SETUP",
                Style::default().fg(ACTIVE_FG).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!("Seed: {}", self.config.seed),
                Style::default().fg(TEAL_ACCENT),
            )),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(header, header_area);

        self.draw_mode_selector(frame, mode_area);
        render_menu_button(
            frame,
            params_btn_area,
            if self.params_expanded {
                "PARAMETERS [-]"
            } else {
                "PARAMETERS [+]"
            },
            self.setup_focus == SetupFocus::ParametersHeader,
        );

        if self.params_expanded {
            self.draw_params_list(frame, params_list_area);
        }

        self.draw_setup_actions(frame, actions_area);

        let hint = Paragraph::new(Line::from(Span::styled(
            "Up/Down: Navigate   Left/Right or 0-9: Edit   r: Reset   Enter: Confirm   Esc: Back",
            Style::default().fg(MUTED_FG),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(hint, footer_area);
    }

    fn draw_mode_selector(&self, frame: &mut Frame, area: Rect) {
        let centered = centered_rect(area, 44);
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(centered);

        let is_focused = self.setup_focus == SetupFocus::Mode;
        let modes = [(TurnMode::Sequential, left), (TurnMode::Simultaneous, right)];

        for (mode, slot) in modes {
            let selected = self.config.turn_mode == mode;
            let border_color = if selected && is_focused {
                ACTIVE_FG
            } else if selected {
                TEAL_ACCENT
            } else {
                MUTED_FG
            };
            let text_style = if selected {
                Style::default().fg(ACTIVE_FG).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(INACTIVE_FG)
            };
            let label = if selected {
                format!("[ {} ]", mode.label())
            } else {
                format!("  {}  ", mode.label())
            };

            let widget = Paragraph::new(Line::from(Span::styled(label, text_style)))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color)),
                );
            frame.render_widget(widget, slot);
        }
    }

    fn draw_params_list(&self, frame: &mut Frame, area: Rect) {
        let box_area = centered_rect(area, 56);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Customizable Parameters ")
            .border_style(Style::default().fg(TEAL_ACCENT));
        let inner = block.inner(box_area);
        frame.render_widget(block, box_area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let lines: Vec<Line<'static>> = PARAM_KINDS
            .iter()
            .enumerate()
            .map(|(idx, kind)| self.render_param_line(idx, *kind))
            .collect();

        frame.render_widget(Paragraph::new(lines), inner);
    }

    fn render_param_line(&self, idx: usize, kind: ParamKind) -> Line<'static> {
        let focused = self.setup_focus == SetupFocus::Param(idx);
        let value_str = match &self.param_edit_buf {
            Some((active_idx, buf)) if *active_idx == idx => format!("{buf}_"),
            _ => kind.get(&self.config.params).to_string(),
        };

        let prefix = if focused { "+ " } else { "  " };
        let prefix_style = if focused {
            Style::default().fg(ACCENT_YELLOW).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(MUTED_FG)
        };
        let label_style = if focused {
            Style::default().fg(ACTIVE_FG).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(INACTIVE_FG)
        };
        let val_style = if focused {
            Style::default().fg(ACCENT_YELLOW).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEAL_ACCENT)
        };

        let arrows = if focused {
            format!("< {value_str:>6} >")
        } else {
            format!("  {value_str:>6}  ")
        };

        Line::from(vec![
            Span::styled(prefix, prefix_style),
            Span::styled(format!("{:<26}", kind.label()), label_style),
            Span::styled(arrows, val_style),
        ])
    }

    fn draw_setup_actions(&self, frame: &mut Frame, area: Rect) {
        let centered = centered_rect(area, 44);
        let [start_area, back_area] =
            Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(centered);

        render_menu_button(frame, start_area, "START GAME", self.setup_focus == SetupFocus::Start);
        render_menu_button(frame, back_area, "BACK", self.setup_focus == SetupFocus::Back);
    }
}

/// Renders a retro menu button: boxed with a yellow `+` sparkle when selected, plain dimmed text otherwise.
fn render_menu_button(frame: &mut Frame, area: Rect, label: &str, selected: bool) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let box_width = u16::try_from(label.len() + 10)
        .unwrap_or(28)
        .max(24)
        .min(area.width.saturating_sub(4));
    let centered = centered_rect(area, box_width);

    if selected {
        let widget = Paragraph::new(Line::from(Span::styled(
            label,
            Style::default().fg(ACTIVE_FG).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ACTIVE_FG)),
        );
        frame.render_widget(widget, centered);

        // Yellow '+' sparkle next to the active button, matching the reference splash screen.
        let sparkle_x = centered.right().saturating_add(2);
        if sparkle_x < area.right() {
            let sparkle_area = Rect::new(sparkle_x, centered.y + 1, 1, 1);
            frame.render_widget(
                Paragraph::new(Span::styled(
                    "+",
                    Style::default().fg(ACCENT_YELLOW).add_modifier(Modifier::BOLD),
                )),
                sparkle_area,
            );
        }
    } else {
        let text_area = Rect::new(centered.x, centered.y + 1, centered.width, 1);
        let widget = Paragraph::new(Line::from(Span::styled(label, Style::default().fg(INACTIVE_FG))))
            .alignment(Alignment::Center);
        frame.render_widget(widget, text_area);
    }
}

/// Returns a horizontally centered sub-rectangle of `area` with width at most `max_width`.
fn centered_rect(area: Rect, max_width: u16) -> Rect {
    let width = max_width.min(area.width);
    let x = area.x + area.width.saturating_sub(width) / 2;
    Rect::new(x, area.y, width, area.height)
}

/// Runs the splash and setup screens on `terminal`, returning the user's choice.
///
/// # Errors
///
/// Returns an [`io::Error`] if terminal rendering or event reading fails.
pub fn run_splash(terminal: &mut ratatui::DefaultTerminal) -> io::Result<SplashOutcome> {
    SplashScreen::new().run(terminal)
}
