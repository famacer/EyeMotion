#![windows_subsystem = "windows"]

mod config;

use std::f64::consts::PI;
use std::time::{Duration, Instant};

use config::UserConfig;
use rand::Rng;
use rodio::{buffer::SamplesBuffer, OutputStream, OutputStreamHandle, Sink, Source};
use sdl2::{
    event::Event,
    keyboard::Keycode,
    keyboard::Scancode,
    mouse::MouseButton,
    pixels::Color,
    rect::{Point, Rect},
    video::FullscreenType,
    render::BlendMode,
};
use rusttype::{Font, Scale, point};

// ==========================================
// --- 常量定义 ---
// ==========================================
const DEFAULT_SIZE: (u32, u32) = (1600, 900); // 默认窗口尺寸
const STAGE_DURATIONS_MS: u64 = 45_000;      // 每个阶段的时长（45秒）
const SAMPLE_RATE: u32 = 44_100;              // 音频采样率
const FONT_DATA: &[u8] = include_bytes!("../assets/fonts/AlumniSansCollegiateOne-Regular.ttf"); // 字体文件数据

// --- 游戏对象 ---

/// 表示游戏中的小球
#[derive(Clone)]
struct Ball {
    screen_w: f64,    // 屏幕宽度
    screen_h: f64,    // 屏幕高度
    radius: f64,      // 小球半径
    pos: (f64, f64),  // 小球位置 (x, y)
    velocity: (f64, f64), // 小球速度 (vx, vy)
}

impl Ball {
    /// 创建一个新的小球实例
    fn new(screen_w: u32, screen_h: u32) -> Self {
        let w = screen_w as f64;
        let h = screen_h as f64;
        Ball {
            screen_w: w,
            screen_h: h,
            radius: w / 40.0,
            pos: (w / 2.0, h / 2.0),
            velocity: (0.0, 0.0),
        }
    }

    /// 重置小球到初始状态
    #[allow(dead_code)]
    fn reset(&mut self, screen_w: u32, screen_h: u32) {
        self.screen_w = screen_w as f64;
        self.screen_h = screen_h as f64;
        self.update_radius();
        self.pos = (self.screen_w / 2.0, self.screen_h / 2.0);
        self.velocity = (0.0, 0.0);
    }
    
    /// 根据屏幕宽度更新小球半径
    fn update_radius(&mut self) {
        self.radius = self.screen_w / 40.0;
    }

    /// 设置小球的速度和方向
    fn set_speed(&mut self, speed: f64, direction: Option<(f64, f64)>) {
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

    /// 更新小球在圆周路径上的运动（用于第5阶段）
    fn update_circular(&mut self, dt: f64, angular_speed: f64) {
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

    /// 更新小球位置并处理碰撞检测
    /// 返回 true 表示发生了碰撞
    fn update(&mut self, dt: f64) -> bool {
        let (vx, vy) = self.velocity;
        self.pos.0 += vx * dt;
        self.pos.1 += vy * dt;

        let mut hit = false;
        // 边界检测与反弹
        if self.pos.0 - self.radius < 0.0 {
            self.pos.0 = self.radius;
            self.velocity.0 = -self.velocity.0;
            hit = true;
        } else if self.pos.0 + self.radius > self.screen_w {
            self.pos.0 = self.screen_w - self.radius;
            self.velocity.0 = -self.velocity.0;
            hit = true;
        }

        if self.pos.1 - self.radius < 0.0 {
            self.pos.1 = self.radius;
            self.velocity.1 = -self.velocity.1;
            hit = true;
        } else if self.pos.1 + self.radius > self.screen_h {
            self.pos.1 = self.screen_h - self.radius;
            self.velocity.1 = -self.velocity.1;
            hit = true;
        }
        hit
    }

    /// 绘制小球
    fn draw<T: sdl2::render::RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>) {
        draw_ball_styled(canvas, self.pos.0 as i32, self.pos.1 as i32, self.radius);
    }
}

// --- 辅助函数 ---

/// 获取指定阶段的小球移动速度
fn stage_speed(stage: i32) -> f64 {
    match stage {
        1 => 450.0,
        2 => 650.0,
        3 => 900.0,
        4 => 1100.0,
        5 => 0.0, // 第 5 阶段使用圆周运动，速度在 update_circular 中处理
        _ => 450.0,
    }
}

/// 获取指定阶段的小球初始运动方向
fn stage_direction(stage: i32) -> (f64, f64) {
    match stage {
        1 | 2 | 3 => {
            let mut rng = rand::thread_rng();
            let angle = rng.gen_range(10.0..80.0) * PI / 180.0;
            (angle.cos(), angle.sin())
        }
        4 => (0.0, 1.0), // 垂直向下
        _ => (1.0, 1.0),
    }
}

/// 生成更自然、更舒适的背景音乐（氛围音）
fn generate_bgm() -> Vec<i16> {
    let duration = 8.0f64; // 较长的循环周期
    let samples = (SAMPLE_RATE as f64 * duration).round() as usize;
    let mut data = Vec::with_capacity(samples * 2);
    
    // 使用多个谐波叠加，产生柔和的氛围感
    let harmonics = [(80.0, 0.2), (120.0, 0.15), (160.0, 0.1)];
    
    for i in 0..samples {
        let t = i as f64 / SAMPLE_RATE as f64;
        let mut s = 0.0;
        
        for &(freq, amp) in harmonics.iter() {
            // 缓慢的振幅调制，增加灵动感
            let modulation = (2.0 * PI * 0.2 * t).sin() * 0.2 + 0.8;
            s += (2.0 * PI * freq * t).sin() * amp * modulation;
        }
        
        // 简单的低通滤波效果（平滑处理）
        if s > 1.0 { s = 1.0; } else if s < -1.0 { s = -1.0; }
        let v = (s * 32767.0) as i16;
        data.push(v); data.push(v);
    }
    data
}

/// 生成清脆、不刺耳的多种碰撞音效
fn generate_bounce_sounds() -> Vec<Vec<i16>> {
    let mut sounds = Vec::new();
    // 提高频率，避开嗡嗡声区域
    let durations = [0.08, 0.1, 0.12];
    let base_freqs = [220.0, 330.0, 440.0, 554.0]; // 使用音阶中的音符频率
    
    for &duration in durations.iter() {
        for &base_freq in base_freqs.iter() {
            let samples = (SAMPLE_RATE as f64 * duration).round() as usize;
            let mut sound_data = Vec::with_capacity(samples * 2);
            for j in 0..samples {
                let t = j as f64 / SAMPLE_RATE as f64;
                let p = t / duration;
                
                // 频率快速下降（敲击感）
                let freq = base_freq * (1.0 - p * 0.5);
                // 使用正弦波 + 少量二次谐波（更清脆）
                let s1 = (2.0 * PI * freq * t).sin() * 0.7;
                let s2 = (2.0 * PI * freq * 2.0 * t).sin() * 0.2;
                
                // 快速衰减的指数包络
                let mut s = (s1 + s2) * (-p * 10.0).exp();
                
                if s > 1.0 { s = 1.0; } else if s < -1.0 { s = -1.0; }
                let v = (s * 32767.0) as i16;
                sound_data.push(v); sound_data.push(v);
            }
            sounds.push(sound_data);
        }
    }
    sounds
}

// --- Main ---

fn main() {
    let sdl_context = sdl2::init().expect("SDL2 Init Failed");
    let video = sdl_context.video().expect("Video Init Failed");
    video.text_input().stop(); // 修复 IME 冲突
    let _audio = sdl_context.audio().expect("Audio Init Failed");

    let mut window = video.window("Eyemotion", DEFAULT_SIZE.0, DEFAULT_SIZE.1)
        .position_centered().resizable().borderless().fullscreen_desktop().build().expect("Window Failed");
    window.raise(); // Force window to top to fix input focus

    let mut canvas = window.into_canvas().present_vsync().build().expect("Canvas Failed");
    canvas.set_blend_mode(BlendMode::Blend);
    let event_pump = sdl_context.event_pump().expect("Events Failed");

    let font = Font::try_from_bytes(FONT_DATA).expect("Font Load Failed");
    let mut user_config = UserConfig::load();
    let (_stream, stream_handle) = OutputStream::try_default().expect("Audio Stream Failed");
    match Sink::try_new(&stream_handle) {
        Ok(sink) => {
             let bgm_data = generate_bgm();
             let bgm_source = SamplesBuffer::new(2, SAMPLE_RATE, bgm_data.clone());
             sink.append(bgm_source.repeat_infinite());
             sink.set_volume(0.5); // Volume 0.5 (Reduced by half)
             
             // 强制开启背景音乐，忽略配置防止意外静音
             user_config.audio.bgm_enabled = true;
             sink.play();

             run_game(sdl_context, canvas, event_pump, font, user_config, sink, stream_handle);
        },
        Err(e) => {
            eprintln!("Audio Sink init failed: {}", e);
             run_game_no_audio(sdl_context, canvas, event_pump, font, user_config);
        }
    }
}

fn run_game(
    sdl_context: sdl2::Sdl, 
    mut canvas: sdl2::render::Canvas<sdl2::video::Window>, 
    mut event_pump: sdl2::EventPump, 
    font: Font<'static>, 
    user_config: UserConfig, 
    sink: Sink,
    stream_handle: OutputStreamHandle
) {
    // 预生成多种碰撞音效
    let bounce_sounds = generate_bounce_sounds();
    let (w, h) = canvas.window().size();
    
    // 初始化游戏对象和状态
    let mut ball = Ball::new(w, h);
    let mut stage = 1;
    let mut stage_elapsed = 0.0; // 当前阶段已进行的时间
    let mut paused = false;      // 暂停状态
    let mut last_time = Instant::now();
    let mut stage5_paused = false;
    let mut stage5_pause_elapsed = 0.0;
    
    // 设置初始速度和方向
    ball.set_speed(stage_speed(stage), Some(stage_direction(stage)));

    let mut is_transitioning = true; // 是否处于阶段转换中
    let mut transition_timer = 3.0;  // 转换倒计时
    let mut is_game_over = false;    // 游戏结束标志
    let mut is_start_screen = true;  // 是否处于开始界面
    let mut mouse_pos = (0, 0);

    'running: loop {
        let now = Instant::now();
        let dt = now.duration_since(last_time).as_secs_f64();
        last_time = now;

        let (curr_w, curr_h) = canvas.output_size().unwrap();
        ball.screen_w = curr_w as f64;
        ball.screen_h = curr_h as f64;
        ball.update_radius();

        // --- 事件处理 ---
        for event in event_pump.poll_iter() {
            match event {
                // 退出事件
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                
                // P 键：切换背景音乐开关
                Event::KeyDown { scancode: Some(Scancode::P), .. } => {
                     user_config.audio.bgm_enabled = !user_config.audio.bgm_enabled;
                     if user_config.audio.bgm_enabled {
                         if sink.empty() {
                             let bgm = generate_bgm();
                             let src = SamplesBuffer::new(2, SAMPLE_RATE, bgm);
                             sink.append(src.repeat_infinite());
                             sink.set_volume(0.5);
                         }
                         sink.play();
                     } else {
                         sink.pause(); 
                     }
                }
                
                // 空格键：开始游戏或切换暂停
                Event::KeyDown { scancode: Some(Scancode::Space), .. } => {
                    if is_start_screen {
                        is_start_screen = false;
                        is_transitioning = true;
                        transition_timer = 3.0;
                    } else if !is_game_over {
                        paused = !paused;
                    }
                }
                
                // R 键：游戏结束时重新开始
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    if is_game_over {
                        stage = 1;
                        stage_elapsed = 0.0;
                        is_game_over = false;
                        is_transitioning = true;
                        transition_timer = 3.0;
                        paused = false;
                        ball.reset(curr_w, curr_h);
                        ball.set_speed(stage_speed(stage), Some(stage_direction(stage)));
                        
                        // 重新开始时如果音乐应该开启则开启
                        if user_config.audio.bgm_enabled {
                            if sink.empty() {
                                let bgm = generate_bgm();
                                let src = SamplesBuffer::new(2, SAMPLE_RATE, bgm);
                                sink.append(src.repeat_infinite());
                                sink.set_volume(0.5);
                            }
                            sink.play();
                        }
                    }
                }
                
                // 鼠标移动
                Event::MouseMotion { x, y, .. } => mouse_pos = (x, y),
                
                // 鼠标点击处理（窗口控制按钮）
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                     let right_edge = curr_w as i32;
                     if y >= 10 && y <= 40 {
                         if x >= right_edge - 120 && x <= right_edge - 90 { 
                             canvas.window_mut().minimize(); 
                         } else if x >= right_edge - 80 && x <= right_edge - 50 { 
                            let win = canvas.window_mut();
                            let fs = win.fullscreen_state();
                            let _ = win.set_fullscreen(if fs == FullscreenType::Off { FullscreenType::Desktop } else { FullscreenType::Off });
                         } else if x >= right_edge - 40 && x <= right_edge - 10 { 
                             break 'running; 
                         }
                      } else {
                          // 点击非按钮区域逻辑同空格键
                          if is_start_screen {
                              is_start_screen = false;
                              is_transitioning = true;
                              transition_timer = 3.0;
                          } else if !is_game_over {
                              paused = !paused;
                          }
                      }
                 }
                _ => {}
            }
        }

        // --- 逻辑更新 ---
        
        // 鼠标光标显示控制
        if is_game_over || is_start_screen || paused { 
            sdl_context.mouse().show_cursor(true); 
        } else { 
            sdl_context.mouse().show_cursor(false); 
        }
        
        if is_transitioning {
            // 阶段转换倒计时
            transition_timer -= dt; 
            if transition_timer <= 0.0 {
                is_transitioning = false;
                stage_elapsed = 0.0; 
            }
        } else if !is_game_over && !is_start_screen && !paused {
            // 正常游戏逻辑更新
            stage_elapsed += dt;
            
            if stage == 5 {
                // 第 5 阶段：特殊圆周运动逻辑
                if stage5_paused {
                    stage5_pause_elapsed += dt;
                    if stage5_pause_elapsed >= 0.3 { stage5_paused = false; }
                } else {
                    let ang_spd = if stage_elapsed < 22.5 { 0.8 } else { -0.8 };
                    ball.update_circular(dt, ang_spd);
                    if stage_elapsed >= 22.5 && stage_elapsed < 22.6 {
                        stage5_paused = true; 
                        stage5_pause_elapsed = 0.0;
                    }
                }
            } else {
                // 普通阶段：直线运动并检测碰撞
                if ball.update(dt) {
                    // 发生碰撞时，随机选择一种音效播放
                    let idx = rand::thread_rng().gen_range(0..bounce_sounds.len());
                    let bounce_src = SamplesBuffer::new(2, SAMPLE_RATE, bounce_sounds[idx].clone());
                    let _ = stream_handle.play_raw(bounce_src.convert_samples());
                }
            }
            
            // 第 4 阶段中途改变方向
            if stage == 4 && stage_elapsed >= 22.5 && stage_elapsed < 22.6 {
                let mut rng = rand::thread_rng();
                let ang = rng.gen_range(10.0..20.0) * PI / 180.0;
                let dir = if rng.gen_bool(0.5) { (ang.sin(), ang.cos()) } else { (ang.sin(), -ang.cos()) };
                ball.set_speed(stage_speed(stage), Some(dir));
            }
            
            // 检查阶段是否结束
            if (stage_elapsed * 1000.0) as u64 > STAGE_DURATIONS_MS {
                if stage >= 5 { 
                    is_game_over = true; 
                    sink.pause(); // 游戏结束暂停音乐
                } else {
                    stage += 1; 
                    is_transitioning = true; 
                    transition_timer = 3.0;
                    ball.set_speed(stage_speed(stage), Some(stage_direction(stage)));
                }
            }
        }

        // --- 渲染 ---
        
        // 绘制背景网格
        let t_size = 80.0;
        let cols = (curr_w as f64 / t_size).round().max(1.0) as u32;
        let rows = (curr_h as f64 / t_size).round().max(1.0) as u32;
        let col_d = Color::RGB(0, 31, 86);
        let col_l = Color::RGB(0, 48, 130);
        
        for r in 0..rows {
            for c in 0..cols {
                let x = (c * curr_w) / cols;
                let y = (r * curr_h) / rows;
                let nx = ((c + 1) * curr_w) / cols;
                let ny = ((r + 1) * curr_h) / rows;
                canvas.set_draw_color(if (c + r) % 2 == 0 { col_d } else { col_l });
                let _ = canvas.fill_rect(Rect::new(x as i32, y as i32, nx - x, ny - y));
            }
        }

        // 绘制窗口控制按钮
        let is_fs = match canvas.window().fullscreen_state() { FullscreenType::Off => false, _ => true };
        draw_window_controls(&mut canvas, mouse_pos, is_fs, curr_w as i32);

        let cx = (curr_w / 2) as i32;
        
        if is_game_over {
            // 绘制游戏结束界面
            let size_go = curr_w as f32 / 8.0;
            let size_restart = curr_w as f32 / 20.0;
            let y_go = (curr_h as f32 * 0.35) as i32;
            let y_restart = (y_go as f32 + size_go * 0.8) as i32;
            draw_text(&mut canvas, &font, "GAME OVER", cx, y_go, size_go, Color::RGB(255, 215, 0));
            draw_text(&mut canvas, &font, "PRESS 'R' TO RESTART", cx, y_restart, size_restart, Color::RGB(200, 200, 200));
        } else if is_start_screen {
            // 绘制开始界面
            let size_title = curr_w as f32 / 8.0;
            let size_start = curr_w as f32 / 20.0;
            let size_footer = curr_w as f32 / 24.0;
            let y_title = (curr_h as f32 * 0.35) as i32;
            let y_start = (y_title as f32 + size_title * 0.5) as i32;
            draw_text(&mut canvas, &font, "EYE MOTION", cx, y_title, size_title, Color::RGB(255, 215, 0));
            draw_text(&mut canvas, &font, "PRESS SPACE TO START", cx, y_start, size_start, Color::RGB(255, 105, 120));
            let y_footer = (curr_h as f32 - size_footer) as i32;
            let bgm_txt = if user_config.audio.bgm_enabled { "ON" } else { "OFF" };
            let footer_text = format!("ESC:EXIT - SPACE:START/PAUSE - P:MUSIC ({})", bgm_txt);
            draw_text(&mut canvas, &font, &footer_text, cx, y_footer, size_footer, Color::RGB(135, 206, 235));
        } else if is_transitioning {
            // 绘制阶段转换提示
            let text = if stage == 5 { "LEVEL R".to_string() } else { format!("LEVEL {}", stage) };
            let size_level = curr_w as f32 / 8.0;
            let y_center_text = (curr_h as f32 * 0.5) as i32;
            draw_text(&mut canvas, &font, &text, cx, y_center_text, size_level, Color::RGB(255, 215, 0));
        } else {
            // 绘制游戏进行中界面
            ball.draw(&mut canvas);
            let rem = (STAGE_DURATIONS_MS as f64 / 1000.0 - stage_elapsed).max(0.0);
            let txt = format!("TIME: {:.0}", rem);
            let size_time = curr_w as f32 / 30.0;
            draw_text_left(&mut canvas, &font, &txt, cx - (size_time * 1.1) as i32, (curr_h as i32) - 80, size_time, Color::RGB(135, 206, 235));
            if paused {
                 let size_pause = curr_w as f32 / 8.0;
                 let y_center_text = (curr_h as f32 * 0.5) as i32;
                 draw_text(&mut canvas, &font, "PAUSED", cx, y_center_text, size_pause, Color::RGB(255, 215, 0));
            }
            // 在游戏中左上角显示音乐状态
            let bgm_st = if user_config.audio.bgm_enabled { "ON" } else { "OFF" };
            draw_text_left(&mut canvas, &font, &format!("MUSIC: {}", bgm_st), 10, 30, 24.0, Color::RGB(200, 200, 200));
        }
        
        canvas.present();
        std::thread::sleep(Duration::from_millis(10));
    }
    let _ = user_config.save();
}

fn run_game_no_audio(
    sdl_context: sdl2::Sdl, 
    mut canvas: sdl2::render::Canvas<sdl2::video::Window>, 
    mut event_pump: sdl2::EventPump, 
    font: Font<'static>, 
    mut user_config: UserConfig
) {
    let (w, h) = canvas.window().size();
    let mut ball = Ball::new(w, h);
    let mut stage = 1;
    let mut stage_elapsed = 0.0;
    let mut paused = false;
    let mut last_time = Instant::now();
    let mut stage5_paused = false;
    let mut stage5_pause_elapsed = 0.0;
    
    ball.set_speed(stage_speed(stage), Some(stage_direction(stage)));

    let mut is_transitioning = true;
    let mut transition_timer = 3.0;
    let mut is_game_over = false;
    let mut is_start_screen = true;
    let mut mouse_pos = (0, 0);

    'running: loop {
        let now = Instant::now();
        let dt = now.duration_since(last_time).as_secs_f64();
        last_time = now;

        let (curr_w, curr_h) = canvas.output_size().unwrap();
        ball.screen_w = curr_w as f64;
        ball.screen_h = curr_h as f64;
        ball.update_radius();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::P), .. } => {},
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    if is_start_screen {
                        is_start_screen = false;
                        is_transitioning = true;
                        transition_timer = 3.0;
                    } else if !is_game_over {
                        paused = !paused;
                    }
                }
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    if is_game_over {
                        stage = 1; stage_elapsed = 0.0; is_game_over = false;
                        is_transitioning = true; transition_timer = 3.0; paused = false;
                        ball.reset(curr_w, curr_h);
                        ball.set_speed(stage_speed(stage), Some(stage_direction(stage)));
                    }
                }
                Event::MouseMotion { x, y, .. } => mouse_pos = (x, y),
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                     let (_, _) = canvas.window().size();
                     let right_edge = curr_w as i32;
                     if y >= 10 && y <= 40 {
                         if x >= right_edge - 120 && x <= right_edge - 90 { canvas.window_mut().minimize(); }
                         else if x >= right_edge - 80 && x <= right_edge - 50 { 
                            let win = canvas.window_mut();
                            let fs = win.fullscreen_state();
                            let _ = win.set_fullscreen(if fs == FullscreenType::Off { FullscreenType::Desktop } else { FullscreenType::Off });
                         }
                          else if x >= right_edge - 40 && x <= right_edge - 10 { break 'running; }
                      } else {
                          // 点击非按钮区域：开始/暂停
                          if is_start_screen {
                              is_start_screen = false;
                              is_transitioning = true;
                              transition_timer = 3.0;
                          } else if !is_game_over {
                              paused = !paused;
                          }
                      }
                 }
                _ => {}
            }
        }

        if is_game_over || is_start_screen { sdl_context.mouse().show_cursor(true); }
        else if is_transitioning || !paused { sdl_context.mouse().show_cursor(false); }
        
        if is_transitioning {
            transition_timer -= dt; 
            if transition_timer <= 0.0 {
                is_transitioning = false;
                stage_elapsed = 0.0; 
            }
        } else if !is_game_over && !is_start_screen && !paused {
            stage_elapsed += dt;
            if stage == 5 {
                if stage5_paused {
                    stage5_pause_elapsed += dt;
                    if stage5_pause_elapsed >= 0.3 { stage5_paused = false; }
                } else {
                    let ang_spd = if stage_elapsed < 22.5 { 0.8 } else { -0.8 };
                    ball.update_circular(dt, ang_spd);
                    if stage_elapsed >= 22.5 && stage_elapsed < 22.6 {
                        stage5_paused = true; stage5_pause_elapsed = 0.0;
                    }
                }
            } else {
                ball.update(dt);
            }
            if stage == 4 && stage_elapsed >= 22.5 && stage_elapsed < 22.6 {
                let mut rng = rand::thread_rng();
                let ang = rng.gen_range(10.0..20.0) * PI / 180.0;
                let dir = if rng.gen_bool(0.5) { (ang.sin(), ang.cos()) } else { (ang.sin(), -ang.cos()) };
                ball.set_speed(stage_speed(stage), Some(dir));
            }
            if (stage_elapsed * 1000.0) as u64 > STAGE_DURATIONS_MS {
                if stage >= 5 { is_game_over = true; }
                else {
                    stage += 1; is_transitioning = true; transition_timer = 3.0;
                    ball.set_speed(stage_speed(stage), Some(stage_direction(stage)));
                }
            }
        }

        // --- Render ---
        draw_background(&mut canvas, curr_w, curr_h);

        let is_fs = match canvas.window().fullscreen_state() { FullscreenType::Off => false, _ => true };
        draw_window_controls(&mut canvas, mouse_pos, is_fs, curr_w as i32);

        let cx = (curr_w / 2) as i32;
        
        if is_game_over {
            let size_go = curr_w as f32 / 8.0;
            let size_restart = curr_w as f32 / 20.0;
            let y_go = (curr_h as f32 * 0.35) as i32;
            let y_restart = (y_go as f32 + size_go * 0.8) as i32;
            draw_text(&mut canvas, &font, "GAME OVER", cx, y_go, size_go, Color::RGB(255, 215, 0));
            draw_text(&mut canvas, &font, "PRESS 'R' TO RESTART", cx, y_restart, size_restart, Color::RGB(200, 200, 200));
        } else if is_start_screen {
            let size_title = curr_w as f32 / 8.0;
            let size_start = curr_w as f32 / 20.0;
            let size_footer = curr_w as f32 / 24.0;
            let y_title = (curr_h as f32 * 0.35) as i32;
            let y_start = (y_title as f32 + size_title * 0.5) as i32;
            draw_text(&mut canvas, &font, "EYE MOTION", cx, y_title, size_title, Color::RGB(255, 215, 0));
            draw_text(&mut canvas, &font, "PRESS SPACE TO START", cx, y_start, size_start, Color::RGB(255, 105, 120));
            let y_footer = (curr_h as f32 - size_footer) as i32;
            draw_text(&mut canvas, &font, "ESC:EXIT - SPACE:START PAUSE - P:MUSIC", cx, y_footer, size_footer, Color::RGB(135, 206, 235));
        } else if is_transitioning {
            let text = if stage == 5 { "LEVEL R".to_string() } else { format!("LEVEL {}", stage) };
            let size_level = curr_w as f32 / 8.0;
            let y_center_text = (curr_h as f32 * 0.5) as i32;
            draw_text(&mut canvas, &font, &text, cx, y_center_text, size_level, Color::RGB(255, 215, 0));
        } else {
            ball.draw(&mut canvas);
            let rem = (STAGE_DURATIONS_MS as f64 / 1000.0 - stage_elapsed).max(0.0);
            let txt = format!("TIME: {:.0}", rem);
            let size_time = curr_w as f32 / 30.0;
            draw_text(&mut canvas, &font, &txt, cx, (curr_h as i32) - 80, size_time, Color::RGB(135, 206, 235));
            if paused {
                 let size_pause = curr_w as f32 / 8.0;
                 // PAUSED文字往上一行
                 let y_center_text = (curr_h as f32 * 0.5) as i32;
                 draw_text(&mut canvas, &font, "PAUSED", cx, y_center_text, size_pause, Color::RGB(255, 215, 0));
            }
        }
        
        canvas.present();
        std::thread::sleep(Duration::from_millis(10));
    }
    let _ = user_config.save();
}

// --- 渲染辅助函数 ---

/// 绘制棋盘格背景
fn draw_background<T: sdl2::render::RenderTarget>(canvas: &mut sdl2::render::Canvas<T>, width: u32, height: u32) {
    let t_size = 80.0;
    let cols = (width as f64 / t_size).round().max(1.0) as u32;
    let rows = (height as f64 / t_size).round().max(1.0) as u32;
    let col_d = Color::RGB(0, 31, 86);
    let col_l = Color::RGB(0, 48, 130);
    
    for r in 0..rows {
        for c in 0..cols {
            let x = (c * width) / cols;
            let y = (r * height) / rows;
            let nx = ((c + 1) * width) / cols;
            let ny = ((r + 1) * height) / rows;
            canvas.set_draw_color(if (c + r) % 2 == 0 { col_d } else { col_l });
            let _ = canvas.fill_rect(Rect::new(x as i32, y as i32, nx - x, ny - y));
        }
    }
}

fn draw_text_left<T: sdl2::render::RenderTarget>(
    canvas: &mut sdl2::render::Canvas<T>,
    font: &Font,
    text: &str,
    x: i32,
    y: i32,
    size: f32,
    color: Color,
) {
    let scale = Scale::uniform(size);
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<_> = font
        .layout(text, scale, point(0.0, v_metrics.ascent))
        .collect();

    let start_x = x as f32; // Left align, no offset

    let height = v_metrics.ascent - v_metrics.descent;
    // 垂直居中：基线位置 = y - 高度的一半 + ascent
    let start_y = y - (height / 2.0) as i32; // Use i32 for consistency

    for glyph in glyphs {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, gv| {
                if gv > 0.0 {
                    let px = start_x as i32 + bb.min.x + gx as i32;
                    let py = start_y + bb.min.y + gy as i32;
                    let alpha = (gv * 255.0) as u8;
                    canvas.set_draw_color(Color::RGBA(color.r, color.g, color.b, alpha));
                    let _ = canvas.draw_point(Point::new(px, py));
                }
            });
        }
    }
}

fn draw_text<T: sdl2::render::RenderTarget>(
    canvas: &mut sdl2::render::Canvas<T>,
    font: &Font,
    text: &str,
    x: i32,
    y: i32,
    size: f32,
    color: Color,
) {
    let scale = Scale::uniform(size);
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<_> = font
        .layout(text, scale, point(0.0, v_metrics.ascent))
        .collect();

    let text_width = glyphs
        .iter()
        .rev()
        .filter_map(|g| g.pixel_bounding_box().map(|b| b.max.x))
        .next()
        .unwrap_or(0) as f32;

    let start_x = x as f32 - text_width / 2.0;

    let height = v_metrics.ascent - v_metrics.descent;
    // 垂直居中：基线位置 = y - 高度的一半 + ascent
    let start_y = y as f32 - height / 2.0;

    for glyph in glyphs {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, gv| {
                if gv > 0.0 {
                    let px = start_x as i32 + bb.min.x + gx as i32;
                    let py = start_y as i32 + bb.min.y + gy as i32;
                    let alpha = (gv * 255.0) as u8;
                    canvas.set_draw_color(Color::RGBA(color.r, color.g, color.b, alpha));
                    let _ = canvas.draw_point(Point::new(px, py));
                }
            });
        }
    }
}

/// 绘制具有样式效果的小球（带渐变和抗锯齿的圆形）
fn draw_ball_styled<T: sdl2::render::RenderTarget>(
    canvas: &mut sdl2::render::Canvas<T>,
    cx: i32,
    cy: i32,
    radius: f64,
) {
    let r = radius as i32;
    
    // 线性渐变颜色
    let color_top_left = (0x10, 0xB4, 0xC3);    // #10B4C3
    let color_bottom_right = (0x11, 0xC5, 0x8C); // #11C58C
    let outline_color = Color::RGB(0x46, 0xE2, 0xD5); // #46E2D5
    
    for dy in -r..=r {
        for dx in -r..=r {
            let dist_sq = (dx * dx + dy * dy) as f64;
            let r_sq = radius * radius;
            
            if dist_sq <= r_sq {
                let dist = dist_sq.sqrt();
                
                // 边缘抗锯齿
                let mut alpha = 255u8;
                if dist > radius - 1.0 {
                    alpha = ((radius - dist) * 255.0) as u8;
                }

                if dist >= radius - 2.0 {
                    // 描边效果
                    let c = outline_color;
                    canvas.set_draw_color(Color::RGBA(c.r, c.g, c.b, alpha));
                    let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
                } else {
                    // 内部渐变填充
                    let gradient_ratio = ((dx + r) as f64 + (dy + r) as f64) / (r as f64 * 4.0);
                    let gradient_ratio = gradient_ratio.max(0.0).min(1.0);
                    
                    let r_val = (color_top_left.0 as f64 + (color_bottom_right.0 as f64 - color_top_left.0 as f64) * gradient_ratio) as u8;
                    let g_val = (color_top_left.1 as f64 + (color_bottom_right.1 as f64 - color_top_left.1 as f64) * gradient_ratio) as u8;
                    let b_val = (color_top_left.2 as f64 + (color_bottom_right.2 as f64 - color_top_left.2 as f64) * gradient_ratio) as u8;
                    
                    canvas.set_draw_color(Color::RGBA(r_val, g_val, b_val, alpha));
                    let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
                }
            }
        }
    }
}

fn draw_window_controls(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, mouse_pos: (i32, i32), is_fs: bool, win_w: i32) {
    let right = win_w;
    let c_min = Color::RGB(0x40, 0xC5, 0xEF);
    let c_max = Color::RGB(0xEB, 0xBF, 0x42);
    let c_close = Color::RGB(0xFC, 0x61, 0x70);
    
    let btns = [
        (right - 120, 10, 30, 30, "MIN", c_min),
        (right - 80, 10, 30, 30, "MAX", c_max),
        (right - 40, 10, 30, 30, "X", c_close),
    ];

    for (x, y, w, h, lbl, col) in btns.iter() {
        let hover = mouse_pos.0 >= *x && mouse_pos.0 <= x + w && mouse_pos.1 >= *y && mouse_pos.1 <= y + h;
        let draw_col = if hover { Color::RGBA(col.r, col.g, col.b, 255) } else { *col };
        canvas.set_draw_color(draw_col);
        let _ = canvas.fill_rect(Rect::new(*x, *y, *w as u32, *h as u32));
        
        canvas.set_draw_color(Color::WHITE);
        match *lbl {
            "MIN" => { let _ = canvas.fill_rect(Rect::new(x + 5, y + h - 10, (w - 10) as u32, 3)); },
            "MAX" => {
                if is_fs {
                    let _ = canvas.draw_rect(Rect::new(x + 10, y + 6, (w - 16) as u32, (h - 16) as u32));
                    let _ = canvas.fill_rect(Rect::new(x + 6, y + 10, (w - 16) as u32, (h - 16) as u32));
                } else {
                    let _ = canvas.draw_rect(Rect::new(x + 6, y + 6, (w - 12) as u32, (h - 12) as u32));
                }
            },
            "X" => {
                for i in -1..=1 {
                    let _ = canvas.draw_line(Point::new(x + 7 + i, y + 7), Point::new(x + w - 7 + i, y + h - 7));
                    let _ = canvas.draw_line(Point::new(x + w - 7 + i, y + 7), Point::new(x + 7 + i, y + h - 7));
                }
            },
            _ => {}
        }
    }
}