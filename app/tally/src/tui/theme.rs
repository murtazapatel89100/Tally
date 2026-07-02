use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders},
};
use rust_decimal::Decimal;

pub struct Theme {
    pub bg: Color,
    pub surface: Color,
    pub text: Color,
    pub subtle: Color,
    pub positive: Color,
    pub negative: Color,
    pub accent: Color,
    pub running: Color,
    pub header: Color,
    pub border: Color,
    pub border_focus: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
}

pub const NORD: Theme = Theme {
    bg: Color::Rgb(46, 52, 64),           // #2e3440  Polar Night 0
    surface: Color::Rgb(59, 66, 82),      // #3b4252  Polar Night 1 (zebra / status)
    text: Color::Rgb(216, 222, 233),      // #d8dee9  Snow Storm 0
    subtle: Color::Rgb(97, 110, 136),     // #616e88  muted comment
    positive: Color::Rgb(163, 190, 140),  // #a3be8c  Aurora green
    negative: Color::Rgb(191, 97, 106),   // #bf616a  Aurora red
    accent: Color::Rgb(235, 203, 139),    // #ebcb8b  Aurora yellow
    running: Color::Rgb(136, 192, 208),   // #88c0d0  Frost 1
    header: Color::Rgb(129, 161, 193),    // #81a1c1  Frost 2
    border: Color::Rgb(76, 86, 106),      // #4c566a  Polar Night 3
    border_focus: Color::Rgb(235, 203, 139), // accent
    selection_bg: Color::Rgb(67, 76, 94), // #434c5e  Polar Night 2
    selection_fg: Color::Rgb(236, 239, 244), // #eceff4  Snow Storm 2
};

impl Theme {
    pub fn base(&self) -> Style {
        Style::default().bg(self.bg).fg(self.text)
    }

    pub fn block<'a>(&self, title: &'a str) -> Block<'a> {
        Block::default()
            .title(Span::styled(
                title,
                Style::default().fg(self.header).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.border))
            .style(self.base())
    }

    pub fn focus_block<'a>(&self, title: &'a str) -> Block<'a> {
        Block::default()
            .title(Span::styled(
                title,
                Style::default().fg(self.accent).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.border_focus))
            .style(self.base())
    }

    pub fn selection(&self) -> Style {
        Style::default()
            .bg(self.selection_bg)
            .fg(self.selection_fg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn amount_style(&self, qty: Decimal) -> Style {
        if qty < Decimal::ZERO {
            Style::default().fg(self.negative)
        } else if qty > Decimal::ZERO {
            Style::default().fg(self.positive)
        } else {
            Style::default().fg(self.subtle)
        }
    }

    pub fn key(&self, k: &str) -> Span<'static> {
        Span::styled(
            k.to_string(),
            Style::default().fg(self.accent).add_modifier(Modifier::BOLD),
        )
    }

    pub fn desc(&self, d: &str) -> Span<'static> {
        Span::styled(d.to_string(), Style::default().fg(self.subtle))
    }
}
