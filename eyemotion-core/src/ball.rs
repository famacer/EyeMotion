use crate::Result;
use rand::Rng;
use std::f64::consts::PI;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Ball {
    pub screen_w: f64,
    pub screen_h: f64,
    pub radius: f64,
    pub pos: (f64, f64),
    pub velocity: (f64, f64),
}

impl Ball {
    pub fn new(screen_w: f64, screen_h: f64) -> Self {
        Ball {
            screen_w,
            screen_h,
            radius: screen_w / 40.0,
            pos: (screen_w / 2.0, screen_h / 2.0),
            velocity: (0.0, 0.0),
        }
    }

    pub fn reset(&mut self, screen_w: f64, screen_h: f64) {
        self.screen_w = screen_w;
        self.screen_h = screen_h;
        self.update_radius();
        self.pos = (self.screen_w / 2.0, self.screen_h / 2.0);
        self.velocity = (0.0, 0.0);
    }

    pub fn update_radius(&mut self) {
        self.radius = self.screen_w / 40.0;
    }

    pub fn set_speed(&mut self, speed: f64, direction: Option<(f64, f64)>) {
        let dir = match direction {
            Some(d) => d,
            None => {
                let angle = rand::thread_rng().gen_range(0.0..(2.0 * PI));
                (angle.cos(), angle.sin())
            }
        };
        let len = (dir.0 * dir.0 + dir.1 * dir.1).sqrt();
        let norm = if len == 0.0 {
            let angle = rand::thread_rng().gen_range(0.0..(2.0 * PI));
            (angle.cos(), angle.sin())
        } else {
            (dir.0 / len, dir.1 / len)
        };
        self.velocity = (norm.0 * speed, norm.1 * speed);
    }

    pub fn update_circular(&mut self, dt: f64, angular_speed: f64) {
        let center_x = self.screen_w / 2.0;
        let center_y = self.screen_h / 2.0;
        let circle_radius = (self.screen_h - self.radius * 2.0) / 2.0;
        let dx = self.pos.0 - center_x;
        let dy = self.pos.1 - center_y;
        let current_angle = dy.atan2(dx);
        let new_angle = current_angle + angular_speed * dt;
        self.pos.0 = center_x + circle_radius * new_angle.cos();
        self.pos.1 = center_y + circle_radius * new_angle.sin();
    }

    pub fn update(&mut self, dt: f64) -> Result<bool> {
        let (vx, vy) = self.velocity;
        self.pos.0 += vx * dt;
        self.pos.1 += vy * dt;
        let r = self.radius;
        let mut bounced = false;

        if self.pos.0 - r <= 0.0 {
            self.pos.0 = r;
            self.velocity.0 = -self.velocity.0;
            bounced = true;
        } else if self.pos.0 + r >= self.screen_w {
            self.pos.0 = self.screen_w - r;
            self.velocity.0 = -self.velocity.0;
            bounced = true;
        }

        if self.pos.1 - r <= 0.0 {
            self.pos.1 = r;
            self.velocity.1 = -self.velocity.1;
            bounced = true;
        } else if self.pos.1 + r >= self.screen_h {
            self.pos.1 = self.screen_h - r;
            self.velocity.1 = -self.velocity.1;
            bounced = true;
        }

        Ok(bounced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ball_creation() {
        let ball = Ball::new(1000.0, 800.0);
        assert_eq!(ball.screen_w, 1000.0);
        assert_eq!(ball.screen_h, 800.0);
        assert_eq!(ball.radius, 25.0);
        assert_eq!(ball.pos, (500.0, 400.0));
    }

    #[test]
    fn test_ball_collision() {
        let mut ball = Ball::new(100.0, 100.0);
        ball.set_speed(1000.0, Some((1.0, 0.0)));
        ball.pos.0 = 95.0;

        let bounced = ball.update(0.01).unwrap();
        assert!(bounced);
    }

    #[test]
    fn test_ball_reset() {
        let mut ball = Ball::new(1000.0, 800.0);
        ball.pos = (100.0, 100.0);
        ball.reset(800.0, 600.0);

        assert_eq!(ball.screen_w, 800.0);
        assert_eq!(ball.screen_h, 600.0);
        assert_eq!(ball.pos, (400.0, 300.0));
    }
}
