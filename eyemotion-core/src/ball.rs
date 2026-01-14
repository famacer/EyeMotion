use crate::Result;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Ball {
    pub screen_w: f64,
    pub screen_h: f64,
    pub radius: f64,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
}

impl Ball {
    pub fn new(screen_w: f64, screen_h: f64) -> Self {
        Ball {
            screen_w,
            screen_h,
            radius: screen_w / 40.0,
            x: screen_w / 2.0,
            y: screen_h / 2.0,
            vx: 0.0,
            vy: 0.0,
        }
    }

    pub fn reset_to_random_pos(&mut self, screen_w: f64, screen_h: f64) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        self.screen_w = screen_w;
        self.screen_h = screen_h;
        self.update_radius();
        let r = self.radius;
        // 随机生成起始点，避开边缘
        self.x = rng.gen_range((r + 50.0)..(self.screen_w - r - 50.0));
        self.y = rng.gen_range((r + 50.0)..(self.screen_h - r - 50.0));
        self.vx = 0.0;
        self.vy = 0.0;
    }

    pub fn reset(&mut self, screen_w: f64, screen_h: f64) {
        self.screen_w = screen_w;
        self.screen_h = screen_h;
        self.update_radius();
        self.x = self.screen_w / 2.0;
        self.y = self.screen_h / 2.0;
        self.vx = 0.0;
        self.vy = 0.0;
    }

    pub fn update_screen_size(&mut self, screen_w: f64, screen_h: f64) {
        // 计算缩放比例
        let scale_x = screen_w / self.screen_w;
        let scale_y = screen_h / self.screen_h;
        
        // 更新屏幕尺寸
        self.screen_w = screen_w;
        self.screen_h = screen_h;
        self.update_radius();
        
        // 按比例缩放球的位置，使其在缩放后保持在相对一致的位置
        self.x *= scale_x;
        self.y *= scale_y;
        
        // 确保球不会因为缩放跑出屏幕边界
        let r = self.radius;
        if self.x < r { self.x = r; }
        if self.x > self.screen_w - r { self.x = self.screen_w - r; }
        if self.y < r { self.y = r; }
        if self.y > self.screen_h - r { self.y = self.screen_h - r; }
    }

    pub fn update_radius(&mut self) {
        self.radius = self.screen_w / 40.0;
    }

    pub fn set_speed(&mut self, speed: f64, direction: Option<(f64, f64)>) {
        if let Some(dir) = direction {
            let mag = (dir.0 * dir.0 + dir.1 * dir.1).sqrt();
            if mag > 0.0 {
                self.vx = (dir.0 / mag) * speed;
                self.vy = (dir.1 / mag) * speed;
            }
        } else {
            let mag = (self.vx * self.vx + self.vy * self.vy).sqrt();
            if mag > 0.0 {
                self.vx = (self.vx / mag) * speed;
                self.vy = (self.vy / mag) * speed;
            }
        }
    }

    pub fn update_circular(&mut self, dt: f64, ang_spd: f64) {
        let center_x = self.screen_w / 2.0;
        let center_y = self.screen_h / 2.0;
        let dx = self.x - center_x;
        let dy = self.y - center_y;
        let radius = (dx * dx + dy * dy).sqrt();
        let mut angle = dy.atan2(dx);

        angle += ang_spd * dt;

        self.x = center_x + radius * angle.cos();
        self.y = center_y + radius * angle.sin();
    }

    pub fn update_circular_with_radius(&mut self, dt: f64, ang_spd: f64, orbit_radius: f64) {
        let center_x = self.screen_w / 2.0;
        let center_y = self.screen_h / 2.0;
        let dx = self.x - center_x;
        let dy = self.y - center_y;
        let mut angle = dy.atan2(dx);

        angle += ang_spd * dt;

        self.x = center_x + orbit_radius * angle.cos();
        self.y = center_y + orbit_radius * angle.sin();
    }

    pub fn update(&mut self, dt: f64) -> Result<bool> {
        self.x += self.vx * dt;
        self.y += self.vy * dt;

        let mut bounced = false;
        let r = self.radius;

        // 边界检测：使用严格的逻辑边界
        if self.x < r {
            self.x = r;
            self.vx = self.vx.abs();
            bounced = true;
        } else if self.x > self.screen_w - r {
            self.x = self.screen_w - r; 
            self.vx = -self.vx.abs();
            bounced = true;
        }

        if self.y < r {
            self.y = r;
            self.vy = self.vy.abs();
            bounced = true;
        } else if self.y > self.screen_h - r {
            self.y = self.screen_h - r;
            self.vy = -self.vy.abs();
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
        assert_eq!((ball.x, ball.y), (500.0, 400.0));
    }

    #[test]
    fn test_ball_collision() {
        let mut ball = Ball::new(100.0, 100.0);
        ball.set_speed(1000.0, Some((1.0, 0.0)));
        ball.x = 95.0;

        let bounced = ball.update(0.01).unwrap();
        assert!(bounced);
    }

    #[test]
    fn test_ball_reset() {
        let mut ball = Ball::new(1000.0, 800.0);
        ball.x = 100.0;
        ball.y = 100.0;
        ball.reset(800.0, 600.0);

        assert_eq!(ball.screen_w, 800.0);
        assert_eq!(ball.screen_h, 600.0);
        assert_eq!((ball.x, ball.y), (400.0, 300.0));
    }
}
