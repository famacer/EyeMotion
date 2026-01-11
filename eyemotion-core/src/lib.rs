pub mod audio_config;
pub mod ball;
pub mod config;
pub mod error;
pub mod events;
pub mod game_state;
pub mod stage_config;
pub mod visual_config;

pub use audio_config::{AudioParams, BGMParams, SFXParams};
pub use ball::Ball;
pub use config::UserConfig;
pub use error::{CoreError, Result};
pub use events::{GameEvent, GameUpdate};
pub use game_state::{stage_direction, stage_speed, GameState};
pub use stage_config::{Axis, MotionType, StageConfig};
pub use visual_config::{BackgroundStyle, BallStyle, Color, Theme, UIStyle};
