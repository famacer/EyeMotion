#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StageConfig {
    pub speed: f64,
    pub angle_range: (f64, f64),
    pub motion_type: MotionType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MotionType {
    Linear { primary_axis: Axis },
    Circular { angular_speed: f64 },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Axis {
    Horizontal,
    Vertical,
    Diagonal,
}

impl Default for StageConfig {
    fn default() -> Self {
        StageConfig {
            speed: 1000.0,
            angle_range: (10.0, 20.0),
            motion_type: MotionType::Linear {
                primary_axis: Axis::Horizontal,
            },
        }
    }
}
