use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Warm cream foreground color matching the retro pixel title aesthetic.
const TITLE_FG: Color = Color::Rgb(245, 250, 225);
/// Teal/cyan drop-shadow color for the layered pixel outline.
const TITLE_SHADOW: Color = Color::Rgb(55, 165, 145);

const LINE_ONE: &str = "RUST IN CASE'S";
const LINE_TWO: &str = "ARCHIPELAGO";

/// Returns a 5-row pixel bitmap for supported title characters.
const fn glyph(ch: char) -> &'static [&'static str] {
    match ch {
        'A' => &[" ## ", "#  #", "####", "#  #", "#  #"],
        'C' => &[" ###", "#   ", "#   ", "#   ", " ###"],
        'E' => &["####", "#   ", "### ", "#   ", "####"],
        'G' => &[" ###", "#   ", "# ##", "#  #", " ###"],
        'H' => &["#  #", "#  #", "####", "#  #", "#  #"],
        'I' => &["###", " # ", " # ", " # ", "###"],
        'L' => &["#   ", "#   ", "#   ", "#   ", "####"],
        'N' => &["#  #", "## #", "# ##", "#  #", "#  #"],
        'O' => &[" ## ", "#  #", "#  #", "#  #", " ## "],
        'P' => &["### ", "#  #", "### ", "#   ", "#   "],
        'R' => &["### ", "#  #", "### ", "# # ", "#  #"],
        'S' => &[" ###", "#   ", " ## ", "   #", "### "],
        'T' => &["#####", "  #  ", "  #  ", "  #  ", "  #  "],
        'U' => &["#  #", "#  #", "#  #", "#  #", " ## "],
        '\'' => &["#", "#", " ", " ", " "],
        _ => &["  ", "  ", "  ", "  ", "  "],
    }
}

/// Builds a 2D boolean pixel grid for `text` (5 rows tall).
fn rasterize_line(text: &str) -> Vec<Vec<bool>> {
    let mut rows = vec![Vec::new(); 5];
    let mut first = true;

    for ch in text.chars() {
        if !first {
            for row in &mut rows {
                row.push(false);
            }
        }
        first = false;

        let g = glyph(ch);
        for (r_idx, pattern) in g.iter().enumerate() {
            for byte in pattern.bytes() {
                rows[r_idx].push(byte == b'#');
            }
        }
    }

    rows
}

/// Renders a single pixel-art line with a 1-pixel offset teal shadow.
fn build_art_lines(text: &str, col_scale: usize) -> Vec<Line<'static>> {
    let fg = rasterize_line(text);
    let height = fg.len();
    let width = fg.first().map_or(0, Vec::len);

    let fg_style = Style::default().fg(TITLE_FG);
    let shadow_style = Style::default().fg(TITLE_SHADOW);
    let block = "█".repeat(col_scale);
    let shadow_block = "░".repeat(col_scale);
    let empty = " ".repeat(col_scale);

    let mut out = Vec::with_capacity(height + 1);
    for y in 0..=height {
        let mut spans = Vec::with_capacity(width + 1);
        for x in 0..=width {
            let is_fg = y < height && x < width && fg[y][x];
            let is_shadow = y > 0 && x > 0 && fg[y - 1][x - 1];

            if is_fg {
                spans.push(Span::styled(block.clone(), fg_style));
            } else if is_shadow {
                spans.push(Span::styled(shadow_block.clone(), shadow_style));
            } else {
                spans.push(Span::raw(empty.clone()));
            }
        }
        out.push(Line::from(spans));
    }
    out
}

/// Returns the preferred vertical height (in terminal rows) for the title at `width`.
#[must_use]
pub fn preferred_height(width: u16) -> u16 {
    if width >= 56 { 13 } else { 3 }
}

/// Renders `"Rust in Case's archipelago"` centered inside `area`.
///
/// Uses a two-line 5x5 pixel-block font with a teal drop-shadow when `area` is wide
/// enough, and falls back to compact styled text on narrow terminals.
pub fn render_title(frame: &mut Frame, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    if area.width >= 112 && area.height >= 13 {
        let mut lines = build_art_lines(LINE_ONE, 2);
        lines.push(Line::default());
        lines.extend(build_art_lines(LINE_TWO, 2));
        frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
        return;
    }

    if area.width >= 56 && area.height >= 13 {
        let mut lines = build_art_lines(LINE_ONE, 1);
        lines.push(Line::default());
        lines.extend(build_art_lines(LINE_TWO, 1));
        frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
        return;
    }

    let compact = vec![
        Line::from(Span::styled(
            "RUST IN CASE'S",
            Style::default().fg(TITLE_FG).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "ARCHIPELAGO",
            Style::default().fg(TITLE_SHADOW).add_modifier(Modifier::BOLD),
        )),
    ];
    frame.render_widget(Paragraph::new(compact).alignment(Alignment::Center), area);
}
