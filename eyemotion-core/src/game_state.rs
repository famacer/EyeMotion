use crate::{Ball, GameEvent, GameUpdate, Result};
use rand::Rng;
use std::f64::consts::PI;

pub const STAGE_DURATIONS_MS: u64 = 45_000;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameState {
    pub ball: Ball,
    pub stage: i32,
    pub stage_elapsed: f64,
    pub paused: bool,
    pub is_transitioning: bool,
    pub transition_timer: f64,
    pub is_game_over: bool,
    pub is_start_screen: bool,
    pub stage5_paused: bool,
    pub stage5_pause_elapsed: f64,
}

impl GameState {
    pub fn new(w: f64, h: f64) -> Self {
        let mut ball = Ball::new(w, h);
        ball.set_speed(stage_speed(1), Some(stage_direction(1)));
        GameState {
            ball,
            stage: 1,
            stage_elapsed: 0.0,
            paused: false,
            is_transitioning: true,
            transition_timer: 3.0,
            is_game_over: false,
            is_start_screen: true,
            stage5_paused: false,
            stage5_pause_elapsed: 0.0,
        }
    }

    pub fn update(&mut self, dt: f64) -> Result<GameUpdate> {
        let mut events = Vec::new();

        if self.is_transitioning {
            self.transition_timer -= dt;
            if self.transition_timer <= 0.0 {
                self.is_transitioning = false;
                self.stage_elapsed = 0.0;
            }
            return Ok(GameUpdate {
                events,
                time_elapsed: self.stage_elapsed,
            });
        }

        if self.is_game_over || self.is_start_screen || self.paused {
            return Ok(GameUpdate {
                events,
                time_elapsed: self.stage_elapsed,
            });
        }

        self.stage_elapsed += dt;

        if self.stage == 5 {
            if self.stage5_paused {
                self.stage5_pause_elapsed += dt;
                if self.stage5_pause_elapsed >= 0.3 {
                    self.stage5_paused = false;
                }
            } else {
                let ang_spd = if self.stage_elapsed < 22.5 { 0.8 } else { -0.8 };
                self.ball.update_circular(dt, ang_spd);
                if self.stage_elapsed >= 22.5 && self.stage_elapsed < 22.6 {
                    self.stage5_paused = true;
                    self.stage5_pause_elapsed = 0.0;
                }
            }
        } else {
            if self.ball.update(dt)? {
                events.push(GameEvent::BallBounced);
            }
        }

        if self.stage == 4 && self.stage_elapsed >= 22.5 && self.stage_elapsed < 22.6 {
            let dir = stage_direction(4);
            self.ball.set_speed(stage_speed(4), Some(dir));
        }

        if (self.stage_elapsed * 1000.0) as u64 > STAGE_DURATIONS_MS {
            if self.stage >= 5 {
                self.is_game_over = true;
                events.push(GameEvent::GameOver);
            } else {
                let prev_stage = self.stage;
                self.stage += 1;
                self.is_transitioning = true;
                self.transition_timer = 3.0;
                self.ball
                    .set_speed(stage_speed(self.stage), Some(stage_direction(self.stage)));
                events.push(GameEvent::StageChanged {
                    from: prev_stage,
                    to: self.stage,
                });
            }
        }

        Ok(GameUpdate {
            events,
            time_elapsed: self.stage_elapsed,
        })
    }

    pub fn reset(&mut self, w: f64, h: f64) {
        self.stage = 1;
        self.stage_elapsed = 0.0;
        self.is_game_over = false;
        self.is_transitioning = true;
        self.transition_timer = 3.0;
        self.paused = false;
        self.ball.reset(w, h);
        self.ball
            .set_speed(stage_speed(1), Some(stage_direction(1)));
    }
}

pub fn stage_direction(stage: i32) -> (f64, f64) {
    let mut rng = rand::thread_rng();
    match stage {
        1 | 5 => {
            let angle_deg = rng.gen_range(10.0..20.0);
            let angle_rad = angle_deg * PI / 180.0;
            if rng.gen_bool(0.5) {
                (angle_rad.cos(), angle_rad.sin())
            } else {
                (-angle_rad.cos(), angle_rad.sin())
            }
        }
        2 => {
            let angle_deg = rng.gen_range(10.0..20.0);
            let angle_rad = angle_deg * PI / 180.0;
            if rng.gen_bool(0.5) {
                (angle_rad.sin(), angle_rad.cos())
            } else {
                (angle_rad.sin(), -angle_rad.cos())
            }
        }
        3 => {
            let angle_deg = rng.gen_range(30.0..40.0);
            let angle_rad = angle_deg * PI / 180.0;
            if rng.gen_bool(0.5) {
                (angle_rad.cos(), angle_rad.sin())
            } else {
                (-angle_rad.cos(), angle_rad.sin())
            }
        }
        4 => {
            let angle_deg = rng.gen_range(10.0..20.0);
            let angle_rad = angle_deg * PI / 180.0;
            if rng.gen_bool(0.5) {
                (angle_rad.cos(), angle_rad.sin())
            } else {
                (-angle_rad.cos(), angle_rad.sin())
            }
        }
        _ => (1.0, 1.0),
    }
}

pub fn stage_speed(stage: i32) -> f64 {
    match stage {
        1 => 1000.0,
        2 => 1200.0,
        3 => 1500.0,
        4 => 2000.0,
        5 => 0.0,
        _ => 1000.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_creation() {
        let state = GameState::new(800.0, 600.0);
        assert_eq!(state.stage, 1);
        assert!(state.is_start_screen);
        assert!(state.is_transitioning);
    }

    #[test]
    fn test_stage_transition() {
        let mut state = GameState::new(800.0, 600.0);
        state.is_start_screen = false;
        state.is_transitioning = false;
        state.stage_elapsed = 46.0;

        let update = state.update(1.0).unwrap();
        assert_eq!(state.stage, 2);
        assert!(update
            .events
            .contains(&GameEvent::StageChanged { from: 1, to: 2 }));
    }

    #[test]
    fn test_game_over() {
        let mut state = GameState::new(800.0, 600.0);
        state.is_start_screen = false;
        state.is_transitioning = false;
        state.stage = 5;
        state.stage_elapsed = 46.0;

        let update = state.update(1.0).unwrap();
        assert!(state.is_game_over);
        assert!(update.events.contains(&GameEvent::GameOver));
    }

    #[test]
    fn test_reset() {
        let mut state = GameState::new(800.0, 600.0);
        state.stage = 5;
        state.stage_elapsed = 100.0;
        state.is_game_over = true;
        state.paused = true;

        state.reset(1000.0, 800.0);

        assert_eq!(state.stage, 1);
        assert_eq!(state.stage_elapsed, 0.0);
        assert!(!state.is_game_over);
        assert!(!state.paused);
        assert!(state.is_transitioning);
        assert_eq!(state.ball.screen_w, 1000.0);
        assert_eq!(state.ball.screen_h, 800.0);
    }
}
