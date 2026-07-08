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
    /// Sign-based +: used for positive amounts in Register
    pub positive: Color,
    /// Sign-based −: used for negative amounts in Register
    pub negative: Color,
    /// Account-type: Assets / Income balances (teal in Tokyo Night)
    pub asset_amt: Color,
    /// Account-type: Expenses / Liabilities balances (amber in Tokyo Night)
    pub expense_amt: Color,
    pub accent: Color,
    /// Running balance column
    pub running: Color,
    pub header: Color,
    pub border: Color,
    pub border_focus: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
}

// ── Tokyo Night ── default ────────────────────────────────────────────────────
pub const TOKYO_NIGHT: Theme = Theme {
    bg:           Color::Rgb(26,  27,  38),  // #1a1b26  deep navy
    surface:      Color::Rgb(30,  32,  48),  // #1e2030  panel / zebra row
    text:         Color::Rgb(192, 202, 245), // #c0caf5  lavender-white
    subtle:       Color::Rgb(86,  95,  137), // #565f89  muted blue-grey
    positive:     Color::Rgb(158, 206, 106), // #9ece6a  bright green (register +)
    negative:     Color::Rgb(247, 118, 142), // #f7768e  red-pink    (register −)
    asset_amt:    Color::Rgb(115, 218, 202), // #73daca  teal    (balance: Assets)
    expense_amt:  Color::Rgb(224, 175, 104), // #e0af68  amber   (balance: Expenses)
    accent:       Color::Rgb(122, 162, 247), // #7aa2f7  blue — keys, focus
    running:      Color::Rgb(125, 207, 255), // #7dcfff  bright cyan — balance column
    header:       Color::Rgb(125, 207, 255), // #7dcfff  column headers
    border:       Color::Rgb(41,  46,  66),  // #292e42  near-invisible dividers
    border_focus: Color::Rgb(224, 175, 104), // #e0af68  amber focus ring (matches image)
    selection_bg: Color::Rgb(42,  46,  70),  // #2a2e46  row / tab highlight
    selection_fg: Color::Rgb(192, 202, 245), // #c0caf5
};

// ── Nord ── `theme = "nord"` ──────────────────────────────────────────────────
pub const NORD: Theme = Theme {
    bg:           Color::Rgb(46,  52,  64),  // #2e3440
    surface:      Color::Rgb(59,  66,  82),  // #3b4252
    text:         Color::Rgb(216, 222, 233), // #d8dee9
    subtle:       Color::Rgb(97,  110, 136), // #616e88
    positive:     Color::Rgb(163, 190, 140), // #a3be8c  Aurora green
    negative:     Color::Rgb(191, 97,  106), // #bf616a  Aurora red
    asset_amt:    Color::Rgb(136, 192, 208), // #88c0d0  Frost — Assets
    expense_amt:  Color::Rgb(235, 203, 139), // #ebcb8b  Aurora yellow — Expenses
    accent:       Color::Rgb(235, 203, 139), // #ebcb8b
    running:      Color::Rgb(136, 192, 208), // #88c0d0
    header:       Color::Rgb(129, 161, 193), // #81a1c1
    border:       Color::Rgb(76,  86,  106), // #4c566a
    border_focus: Color::Rgb(235, 203, 139),
    selection_bg: Color::Rgb(67,  76,  94),  // #434c5e
    selection_fg: Color::Rgb(236, 239, 244), // #eceff4
};

// ── Light ── `theme = "light"` ────────────────────────────────────────────────
pub const LIGHT: Theme = Theme {
    bg:           Color::Rgb(236, 239, 244),
    surface:      Color::Rgb(229, 233, 240),
    text:         Color::Rgb(46,  52,  64),
    subtle:       Color::Rgb(76,  86,  106),
    positive:     Color::Rgb(88,  130, 70),
    negative:     Color::Rgb(160, 65,  72),
    asset_amt:    Color::Rgb(60,  110, 130),
    expense_amt:  Color::Rgb(150, 115, 50),
    accent:       Color::Rgb(150, 115, 50),
    running:      Color::Rgb(60,  110, 130),
    header:       Color::Rgb(74,  110, 160),
    border:       Color::Rgb(163, 190, 140),
    border_focus: Color::Rgb(150, 115, 50),
    selection_bg: Color::Rgb(216, 222, 233),
    selection_fg: Color::Rgb(46,  52,  64),
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
                Style::default().fg(self.border_focus).add_modifier(Modifier::BOLD),
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

    /// Sign-based: used for Register amounts and net-worth display.
    pub fn amount_style(&self, qty: Decimal) -> Style {
        if qty < Decimal::ZERO {
            Style::default().fg(self.negative)
        } else if qty > Decimal::ZERO {
            Style::default().fg(self.positive)
        } else {
            Style::default().fg(self.subtle)
        }
    }

    /// Account-type-based: used in Balances view.
    /// Assets/Income → asset_amt (teal), Expenses/Liabilities → expense_amt (amber).
    pub fn account_amount_style(&self, top_level: &str, qty: Decimal) -> Style {
        match top_level {
            "Assets" | "Income" => {
                if qty == Decimal::ZERO {
                    Style::default().fg(self.subtle)
                } else {
                    Style::default().fg(self.asset_amt)
                }
            }
            "Liabilities" | "Expenses" => {
                if qty == Decimal::ZERO {
                    Style::default().fg(self.subtle)
                } else {
                    Style::default().fg(self.expense_amt)
                }
            }
            "Equity" => Style::default().fg(self.subtle),
            _ => self.amount_style(qty),
        }
    }

    pub fn key(&self, k: &str) -> Span<'static> {
        Span::styled(
            k.to_string(),
            Style::default().fg(self.text).add_modifier(Modifier::BOLD),
        )
    }

    pub fn desc(&self, d: &str) -> Span<'static> {
        Span::styled(d.to_string(), Style::default().fg(self.subtle))
    }
}
