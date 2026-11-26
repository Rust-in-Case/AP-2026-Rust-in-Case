use contractors::shared::Params;
use crossterm::event::KeyCode;
use ratatui::{Terminal, backend::TestBackend};

use crate::config::{PARAM_KINDS, ParamKind, TurnMode};
use crate::splash::{SplashOutcome, SplashScreen};

#[test]
fn main_menu_quit_returns_quit_outcome() {
    let mut splash = SplashScreen::new();

    // Move from NEW GAME -> FROM SEED -> QUIT, then press Enter.
    splash.handle_key(KeyCode::Down);
    splash.handle_key(KeyCode::Down);
    splash.handle_key(KeyCode::Enter);

    assert_eq!(splash.outcome(), Some(SplashOutcome::Quit));
}

#[test]
fn new_game_setup_toggles_mode_and_edits_parameters() {
    let mut splash = SplashScreen::new();

    // Select NEW GAME.
    splash.handle_key(KeyCode::Enter);
    // Toggle mode to SIMULTANEOUS.
    splash.handle_key(KeyCode::Right);
    // Move to PARAMETERS header, then first parameter (Archipelago Radius: default 12).
    splash.handle_key(KeyCode::Down);
    splash.handle_key(KeyCode::Down);
    // Increment radius twice (12 -> 14).
    splash.handle_key(KeyCode::Right);
    splash.handle_key(KeyCode::Right);

    // Move down through remaining 9 parameters to START GAME.
    for _ in 0..10 {
        splash.handle_key(KeyCode::Down);
    }
    splash.handle_key(KeyCode::Enter);

    match splash.outcome() {
        Some(SplashOutcome::Start(cfg)) => {
            assert_eq!(cfg.turn_mode, TurnMode::Simultaneous);
            assert_eq!(cfg.params.archipelago_radius, 14);
        }
        other => panic!("expected Start outcome, got {other:?}"),
    }
}

#[test]
fn from_seed_parses_custom_seed_and_starts() {
    let mut splash = SplashScreen::new();

    // Select FROM SEED.
    splash.handle_key(KeyCode::Down);
    splash.handle_key(KeyCode::Enter);

    // Type seed "42069" and confirm.
    for ch in ['4', '2', '0', '6', '9'] {
        splash.handle_key(KeyCode::Char(ch));
    }
    splash.handle_key(KeyCode::Enter);

    // Navigate directly to START GAME (Mode -> Params header -> 10 params -> Start).
    for _ in 0..12 {
        splash.handle_key(KeyCode::Down);
    }
    splash.handle_key(KeyCode::Enter);

    match splash.outcome() {
        Some(SplashOutcome::Start(cfg)) => {
            assert_eq!(cfg.seed, 42069);
            assert_eq!(cfg.turn_mode, TurnMode::Sequential);
        }
        other => panic!("expected Start outcome, got {other:?}"),
    }
}

#[test]
fn param_kind_clamps_within_bounds() {
    let mut params = Params::default();
    ParamKind::ArchipelagoRadius.set(&mut params, 0);
    assert_eq!(params.archipelago_radius, 1);

    ParamKind::IslandCount.set(&mut params, 999);
    assert_eq!(params.island_count, u8::MAX);

    for kind in PARAM_KINDS {
        let val = kind.get(&params);
        let (min, max, _) = kind.bounds();
        assert!(val >= min && val <= max);
    }
}

#[test]
fn splash_renders_title_and_buttons_without_version_or_music() {
    let splash = SplashScreen::new();
    let backend = TestBackend::new(120, 36);
    let mut terminal = Terminal::new(backend).expect("test terminal");

    terminal.draw(|frame| splash.draw(frame)).expect("draw succeeds");

    let buffer = terminal.backend().buffer();
    let mut text = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            text.push_str(buffer[(x, y)].symbol());
        }
        text.push('\n');
    }

    assert!(text.contains("NEW GAME"));
    assert!(text.contains("FROM SEED"));
    assert!(text.contains("QUIT"));
    assert!(!text.contains("Version"));
    assert!(!text.contains("Music"));
}
