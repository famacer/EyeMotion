#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Theme {
    pub background: BackgroundStyle,
    pub ball: BallStyle,
    pub ui: UIStyle,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackgroundStyle {
    pub grid_color_dark: Color,
    pub grid_color_light: Color,
    pub grid_size: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BallStyle {
    pub gradient_start: Color,
    pub gradient_end: Color,
    pub outline_color: Color,
    pub radius_ratio: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UIStyle {
    pub title_color: Color,
    pub subtitle_color: Color,
    pub stats_color: Color,
    pub button_color: Color,
    pub button_hover_color: Color,
    pub background_color: Color,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            background: BackgroundStyle {
                grid_color_dark: Color { r: 0, g: 31, b: 86 },
                grid_color_light: Color {
                    r: 0,
                    g: 48,
                    b: 130,
                },
                grid_size: 80.0,
            },
            ball: BallStyle {
                gradient_start: Color {
                    r: 16,
                    g: 180,
                    b: 195,
                },
                gradient_end: Color {
                    r: 17,
                    g: 197,
                    b: 140,
                },
                outline_color: Color {
                    r: 70,
                    g: 226,
                    b: 213,
                },
                radius_ratio: 1.0 / 40.0,
            },
            ui: UIStyle {
                title_color: Color {
                    r: 255,
                    g: 215,
                    b: 0,
                },
                subtitle_color: Color {
                    r: 255,
                    g: 105,
                    b: 120,
                },
                stats_color: Color {
                    r: 135,
                    g: 206,
                    b: 235,
                },
                button_color: Color {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                button_hover_color: Color {
                    r: 240,
                    g: 240,
                    b: 240,
                },
                background_color: Color { r: 0, g: 31, b: 86 },
            },
        }
    }
}
