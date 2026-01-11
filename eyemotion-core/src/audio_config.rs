#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioParams {
    pub bgm: BGMParams,
    pub sfx: SFXParams,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BGMParams {
    pub enabled: bool,
    pub volume: f32,
    pub chords: Vec<Vec<f64>>,
    pub melody: Vec<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SFXParams {
    pub enabled: bool,
    pub volume: f32,
    pub bounce_freq_start: f64,
    pub bounce_freq_end: f64,
    pub bounce_duration: f64,
}

impl Default for AudioParams {
    fn default() -> Self {
        AudioParams {
            bgm: BGMParams {
                enabled: true,
                volume: 0.15,
                chords: vec![
                    vec![130.81, 261.63, 329.63, 392.00, 493.88],
                    vec![220.00, 261.63, 329.63, 392.00],
                    vec![174.61, 261.63, 329.63, 349.23],
                    vec![196.00, 493.88, 293.66, 349.23],
                ],
                melody: vec![
                    261.63, 329.63, 392.00, 493.88, 440.00, 392.00, 329.63, 293.66,
                ],
            },
            sfx: SFXParams {
                enabled: true,
                volume: 0.8,
                bounce_freq_start: 150.0,
                bounce_freq_end: 75.0,
                bounce_duration: 0.08,
            },
        }
    }
}
