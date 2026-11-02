//! Color palettes and the stylesheet generated from them.
//!
//! Tile colors are theme independent and painted by the board widget; the
//! palette drives everything around the board (window, header bar, score
//! pills, buttons, overlay).

use adw::ColorScheme;

/// Colors for one visual mode.
pub struct Palette {
    pub name: &'static str,
    pub label: &'static str,
    pub scheme: ColorScheme,
    pub window_bg: &'static str,
    pub fg: &'static str,
    pub board_bg: &'static str,
    pub empty_tile: &'static str,
    pub scorebox_bg: &'static str,
    pub caption_fg: &'static str,
    pub value_fg: &'static str,
    pub button_bg: &'static str,
    pub button_hover: &'static str,
    pub button_fg: &'static str,
    pub overlay_bg: &'static str,
    pub overlay_fg: &'static str,
    pub hint_fg: &'static str,
    pub subtitle_fg: &'static str,
}

/// Neutral light mode.
pub const LIGHT: Palette = Palette {
    name: "light",
    label: "Light",
    scheme: ColorScheme::ForceLight,
    window_bg: "#f5f5f4",
    fg: "#474747",
    board_bg: "#b3b3ab",
    empty_tile: "rgba(255, 255, 255, 0.45)",
    scorebox_bg: "#8f8f87",
    caption_fg: "rgba(255, 255, 255, 0.7)",
    value_fg: "#ffffff",
    button_bg: "#6f6f67",
    button_hover: "#5f5f58",
    button_fg: "#ffffff",
    overlay_bg: "rgba(245, 245, 244, 0.88)",
    overlay_fg: "#474747",
    hint_fg: "rgba(71, 71, 71, 0.75)",
    subtitle_fg: "rgba(71, 71, 71, 0.8)",
};

/// The classic play2048.co cream look.
pub const SEPIA: Palette = Palette {
    name: "sepia",
    label: "Sepia",
    scheme: ColorScheme::ForceLight,
    window_bg: "#faf8ef",
    fg: "#776e65",
    board_bg: "#bbada0",
    empty_tile: "rgba(238, 228, 218, 0.35)",
    scorebox_bg: "#bbada0",
    caption_fg: "#eee4da",
    value_fg: "#ffffff",
    button_bg: "#8f7a66",
    button_hover: "#7e6a57",
    button_fg: "#f9f6f2",
    overlay_bg: "rgba(250, 248, 239, 0.9)",
    overlay_fg: "#776e65",
    hint_fg: "rgba(119, 110, 101, 0.75)",
    subtitle_fg: "rgba(119, 110, 101, 0.85)",
};

/// Warm dark mode.
pub const DARK: Palette = Palette {
    name: "dark",
    label: "Dark",
    scheme: ColorScheme::ForceDark,
    window_bg: "#1d1c1a",
    fg: "#e9e4da",
    board_bg: "#37342e",
    empty_tile: "rgba(238, 228, 218, 0.12)",
    scorebox_bg: "#37342e",
    caption_fg: "rgba(238, 228, 218, 0.6)",
    value_fg: "#eee4da",
    button_bg: "#7f6f5f",
    button_hover: "#8e7e6d",
    button_fg: "#f9f6f2",
    overlay_bg: "rgba(29, 28, 26, 0.85)",
    overlay_fg: "#e9e4da",
    hint_fg: "rgba(233, 228, 218, 0.65)",
    subtitle_fg: "rgba(233, 228, 218, 0.8)",
};

/// All modes in drop-down order.
pub const PALETTES: [&Palette; 3] = [&LIGHT, &SEPIA, &DARK];

/// The stylesheet for a palette.
pub fn css(p: &Palette) -> String {
    format!(
        "
window {{
    background-color: {window_bg};
    color: {fg};
}}
headerbar {{
    background-color: {window_bg};
    color: {fg};
}}
.game-title {{
    font-size: 52px;
    font-weight: 800;
}}
.subtitle {{
    font-size: 14px;
    color: {subtitle_fg};
}}
.scorebox {{
    background-color: {scorebox_bg};
    border-radius: 6px;
    padding: 4px 20px;
}}
.scorebox .caption {{
    font-size: 11px;
    font-weight: bold;
    color: {caption_fg};
}}
.scorebox .value {{
    font-size: 20px;
    font-weight: bold;
    color: {value_fg};
}}
button.new-game {{
    background-color: {button_bg};
    color: {button_fg};
    font-weight: bold;
    font-size: 15px;
    padding: 9px 22px;
    border-radius: 6px;
}}
button.new-game:hover {{
    background-color: {button_hover};
}}
.hint {{
    font-size: 12px;
    color: {hint_fg};
}}
.overlay {{
    background-color: {overlay_bg};
    border-radius: 10px;
    padding: 24px 36px;
}}
.overlay-title {{
    font-size: 26px;
    font-weight: bold;
    color: {overlay_fg};
}}
",
        window_bg = p.window_bg,
        fg = p.fg,
        subtitle_fg = p.subtitle_fg,
        scorebox_bg = p.scorebox_bg,
        caption_fg = p.caption_fg,
        value_fg = p.value_fg,
        button_bg = p.button_bg,
        button_hover = p.button_hover,
        button_fg = p.button_fg,
        hint_fg = p.hint_fg,
        overlay_bg = p.overlay_bg,
        overlay_fg = p.overlay_fg,
    )
}
