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
            transition_timer: 3.0, // 所有关卡统一为 3.0s 倒计时
            is_game_over: false,
            is_start_screen: true,
            stage5_paused: false,
            stage5_pause_elapsed: 0.0,
        }
    }

    pub fn update(&mut self, dt: f64) -> Result<GameUpdate> {
        let mut events = Vec::new();

        if self.is_game_over || self.is_start_screen || self.paused {
            return Ok(GameUpdate {
                events,
                time_elapsed: self.stage_elapsed,
            });
        }

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

        self.stage_elapsed += dt;

        if self.stage == 5 {
            if self.stage5_paused {
                self.stage5_pause_elapsed += dt;
                if self.stage5_pause_elapsed >= 0.3 {
                    self.stage5_paused = false;
                }
            } else {
                // Stage 5 小球速度
                let ang_spd = if self.stage_elapsed < 22.5 { 1.2 } else { -1.2 };
                
                // 修正 Stage 5 边界：距离边界 16px (相对)
                let r = self.ball.radius;
                let margin = 16.0;
                let orbit_radius = (self.ball.screen_h - 2.0 * (r + margin)) / 2.0;
                self.ball.update_circular_with_radius(dt, ang_spd, orbit_radius);
                
                // 边界检查确保不越界
                let min_x = r + margin;
                let max_x = self.ball.screen_w - (r + margin);
                let min_y = r + margin;
                let max_y = self.ball.screen_h - (r + margin);
                
                if self.ball.y < min_y { self.ball.y = min_y; }
                else if self.ball.y > max_y { self.ball.y = max_y; }
                if self.ball.x < min_x { self.ball.x = min_x; }
                else if self.ball.x > max_x { self.ball.x = max_x; }

                if self.stage_elapsed >= 22.5 && self.stage_elapsed < 22.6 {
                    self.stage5_paused = true;
                    self.stage5_pause_elapsed = 0.0;
                }
            }
        } else if self.stage == 4 {
            // Stage 4: 遵循“碰到边界弹开”的原则
            if self.ball.update(dt)? {
                events.push(GameEvent::BallBounced);
                // 为了让运动更“无规律一些”，我们在反弹时加入随机角度偏移
                let mut rng = rand::thread_rng();
                let nudge = rng.gen_range(-0.1..0.1); // 增加偏移范围到约 +/- 6 度
                let current_speed = (self.ball.vx * self.ball.vx + self.ball.vy * self.ball.vy).sqrt();
                let current_angle = self.ball.vy.atan2(self.ball.vx);
                let new_angle = current_angle + nudge;
                self.ball.vx = new_angle.cos() * current_speed;
                self.ball.vy = new_angle.sin() * current_speed;
            }
        } else if self.stage == 3 {
            // Stage 3: 前 22.5s 垂直居中运动，后 22.5s 水平居中运动
            let is_first_half = self.stage_elapsed < 22.5;
            let base_speed = stage_speed(3);
            
            if is_first_half {
                // 垂直运动：速度为水平速度的 0.7 倍 (x0.7)，位置强制居中
                let v_speed = base_speed * 0.7;
                self.ball.vx = 0.0;
                self.ball.x = self.ball.screen_w / 2.0;
                self.ball.vy = if self.ball.vy >= 0.0 { v_speed } else { -v_speed };
            } else {
                // 水平运动：标准速度 (x1.0)，位置强制居中
                let h_speed = base_speed;
                self.ball.vy = 0.0;
                self.ball.y = self.ball.screen_h / 2.0;
                self.ball.vx = if self.ball.vx >= 0.0 { h_speed } else { -h_speed };
            }

            if self.ball.update(dt)? {
                events.push(GameEvent::BallBounced);
            }
        } else if self.stage == 2 {
            // Stage 2: 仅保留垂直运动
            if self.ball.update(dt)? {
                events.push(GameEvent::BallBounced);
            }
        } else {
            // Stage 1: 标准碰撞反弹
            if self.ball.update(dt)? {
                events.push(GameEvent::BallBounced);
            }
        }

        if (self.stage_elapsed * 1000.0) as u64 > STAGE_DURATIONS_MS {
            self.next_stage(&mut events);
        }

        Ok(GameUpdate {
            events,
            time_elapsed: self.stage_elapsed,
        })
    }

    pub fn next_stage(&mut self, events: &mut Vec<GameEvent>) {
        if self.stage >= 5 {
            self.is_game_over = true;
            events.push(GameEvent::GameOver);
        } else {
            let prev_stage = self.stage;
            self.stage += 1;
            self.is_transitioning = true;
            self.transition_timer = 3.0; // 关卡间倒计时维持 3.0s
            
            // 每一关开始时，将小球重置到随机位置，打破起始点与上一关终点的联系
            self.ball.reset_to_random_pos(self.ball.screen_w, self.ball.screen_h);
            
            self.ball
                .set_speed(stage_speed(self.stage), Some(stage_direction(self.stage)));
            events.push(GameEvent::StageChanged {
                from: prev_stage,
                to: self.stage,
            });
        }
    }

    pub fn prev_stage(&mut self, events: &mut Vec<GameEvent>) {
        if self.stage > 1 {
            let prev_stage = self.stage;
            self.stage -= 1;
            self.is_transitioning = true;
            self.transition_timer = 3.0;
            
            // 切换回上一关时也随机重置位置
            self.ball.reset_to_random_pos(self.ball.screen_w, self.ball.screen_h);
            
            self.ball
                .set_speed(stage_speed(self.stage), Some(stage_direction(self.stage)));
            events.push(GameEvent::StageChanged {
                from: prev_stage,
                to: self.stage,
            });
        }
    }

    pub fn reset(&mut self, w: f64, h: f64) {
        self.stage = 1;
        self.stage_elapsed = 0.0;
        self.is_game_over = false;
        self.is_transitioning = true;
        self.transition_timer = 3.0; // 重置也统一为 3.0s
        self.paused = false;
        self.ball.reset(w, h);
        self.ball
            .set_speed(stage_speed(1), Some(stage_direction(1)));
    }

    pub fn resize(&mut self, w: f64, h: f64) {
        self.ball.update_screen_size(w, h);
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
            // Stage 2: 垂直方向带有一点随机偏角 (5-15度)
            let angle_deg = rng.gen_range(5.0..15.0);
            let angle_rad = angle_deg * PI / 180.0;
            let vx = if rng.gen_bool(0.5) { angle_rad.sin() } else { -angle_rad.sin() };
            let vy = if rng.gen_bool(0.5) { angle_rad.cos() } else { -angle_rad.cos() };
            (vx, vy)
        }
        4 => {
            // 为 Stage 4 提供更随机的初始角度，不再仅限于垂直附近
            let angle_deg = rng.gen_range(20.0..70.0); // 20-70度，更像对角线
            let angle_rad = angle_deg * PI / 180.0;
            if rng.gen_bool(0.5) {
                (angle_rad.sin(), angle_rad.cos())
            } else {
                (angle_rad.sin(), -angle_rad.cos())
            }
        }
        3 => {
            // Stage 3 初始强制垂直向上或向下
            if rng.gen_bool(0.5) {
                (0.0, 1.0)
            } else {
                (0.0, -1.0)
            }
        }
        _ => (1.0, 1.0),
    }
}

pub fn stage_speed(stage: i32) -> f64 {
    match stage {
        1 => 1000.0,
        2 => 1100.0, // 从 1250.0 降低到 1100.0
        3 => 1625.0,
        4 => 1500.0, // 从 1800.0 降低到 1500.0
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
