use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

/// Returns true if the credential field should be masked in the TUI.
/// Applies to keys containing SENHA, PASSWORD, or SECRET (case-insensitive).
/// Login-style keys (CHAVE, USUARIO, LOGIN, API_KEY, TOKEN, etc.) are NOT masked.
pub(super) fn is_masked_field(key: &str) -> bool {
    let u = key.to_uppercase();
    u.contains("SENHA") || u.contains("PASSWORD") || u.contains("SECRET")
}

// ─── Tokyo Night palette ──────────────────────────────────────────────────────

pub(super) const TN_FG: Color     = Color::Rgb(192, 202, 245);
pub(super) const TN_MUTED: Color  = Color::Rgb(86, 95, 137);
pub(super) const TN_BLUE: Color   = Color::Rgb(122, 162, 247);
pub(super) const TN_PURPLE: Color = Color::Rgb(187, 154, 247);
pub(super) const TN_CYAN: Color   = Color::Rgb(125, 207, 255);
pub(super) const TN_GREEN: Color  = Color::Rgb(158, 206, 106);
pub(super) const TN_RED: Color    = Color::Rgb(247, 118, 142);
pub(super) const TN_YELLOW: Color = Color::Rgb(224, 175, 104);
pub(super) const TN_ORANGE: Color = Color::Rgb(255, 158, 100);
pub(super) const TN_SEL: Color    = Color::Rgb(54, 74, 130);

// ─── Span helpers ─────────────────────────────────────────────────────────────

pub(super) fn key(label: &'static str) -> Span<'static> {
    Span::styled(label, Style::default().fg(TN_BLUE).add_modifier(Modifier::BOLD))
}

pub(super) fn desc(label: &'static str) -> Span<'static> {
    Span::styled(label, Style::default().fg(TN_MUTED))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_senha_keyword() {
        assert!(is_masked_field("SENHA"));
        assert!(is_masked_field("minha_senha"));
        assert!(is_masked_field("DB_PASSWORD"));
        assert!(is_masked_field("api_secret"));
    }

    #[test]
    fn does_not_mask_login_style_keys() {
        assert!(!is_masked_field("CHAVE"));
        assert!(!is_masked_field("USUARIO"));
        assert!(!is_masked_field("LOGIN"));
        assert!(!is_masked_field("API_KEY"));
        assert!(!is_masked_field("TOKEN"));
    }
}
