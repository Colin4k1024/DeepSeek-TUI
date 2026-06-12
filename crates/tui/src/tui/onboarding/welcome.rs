//! Welcome screen content for onboarding.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::palette;

const WHALE_LOGO: &[&str] = &[
    r"   ╭──────────────────────────────────────────╮",
    r"   │                                          │",
    r"   │      ~~              .--'                 │",
    r"   │       '--.----.---.'    🐋                │",
    r"   │           '--------'                     │",
    r"   │                                          │",
    r"   │    ██████╗ ██████╗ ██████╗ ███████╗      │",
    r"   │   ██╔════╝██╔═══██╗██╔══██╗██╔════╝      │",
    r"   │   ██║     ██║   ██║██║  ██║█████╗        │",
    r"   │   ██║     ██║   ██║██║  ██║██╔══╝        │",
    r"   │   ╚██████╗╚██████╔╝██████╔╝███████╗      │",
    r"   │    ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝      │",
    r"   │     W  H  A  L  E                        │",
    r"   │                                          │",
    r"   ╰──────────────────────────────────────────╯",
];

pub fn lines() -> Vec<Line<'static>> {
    let mut result: Vec<Line<'static>> = Vec::new();

    for line in WHALE_LOGO {
        result.push(Line::from(Span::styled(
            *line,
            Style::default()
                .fg(palette::DEEPSEEK_BLUE)
                .add_modifier(Modifier::BOLD),
        )));
    }

    result.push(Line::from(""));
    result.push(Line::from(Span::styled(
        format!("   Version {}", env!("CARGO_PKG_VERSION")),
        Style::default().fg(palette::TEXT_MUTED),
    )));
    result.push(Line::from(""));
    result.push(Line::from(Span::styled(
        "   AI-powered terminal workspace for deep coding sessions.",
        Style::default().fg(palette::TEXT_PRIMARY),
    )));
    result.push(Line::from(Span::styled(
        "   Press Enter to continue.  Ctrl+C exits.",
        Style::default().fg(palette::TEXT_MUTED),
    )));

    result
}
