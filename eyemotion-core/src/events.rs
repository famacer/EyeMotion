#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum GameEvent {
    BallBounced,
    StageChanged { from: i32, to: i32 },
    StageCompleted { stage: i32 },
    GameOver,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GameUpdate {
    pub events: Vec<GameEvent>,
    pub time_elapsed: f64,
}
