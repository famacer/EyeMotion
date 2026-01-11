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
const DEFAULT_SIZE: (u32, u32) = (1600, 900);
const STAGE_DURATIONS_MS: u64 = 45_000;
const SAMPLE_RATE: u32 = 44_100;
const FONT_DATA: &[u8] = include_bytes!("../assets/fonts/AlumniSansCollegiateOne-Regular.ttf");

// --- 游戏对象 ---

#[derive(Clone)]
struct Ball {
    screen_w: f64,
    screen_h: f64,
    radius: f64,
    pos: (f64, f64),
    velocity: (f64, f64),
}

impl Ball {
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

    #[allow(dead_code)]
    fn reset(&mut self, screen_w: u32, screen_h: u32) {
        self.screen_w = screen_w as f64;
        self.screen_h = screen_h as f64;
        self.update_radius();
        self.pos = (self.screen_w / 2.0, self.screen_h / 2.0);
        self.velocity = (0.0, 0.0);
    }
    
    fn update_radius(&mut self) {
        self.radius = self.screen_w / 40.0;
    }

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

    fn update(&mut self, dt: f64) -> bool {
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

        if self.pos.1 - r<= 0.0 {
            self.pos.1 = r;
            self.velocity.1 = -self.velocity.1;
            bounced = true;
        } else if self.pos.1 + r >= self.screen_h {
            self.pos.1 = self.screen_h - r;
            self.velocity.1 = -self.velocity.1;
            bounced = true;
        }

        bounced
    }

    fn draw<T: sdl2::render::RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>) {
        let x = self.pos.0 as i32;
        let y = self.pos.1 as i32;
        let r = self.radius; 
        draw_ball_styled(canvas, x, y, r);
    }
}

// --- 辅助函数 ---

fn stage_speed(stage: i32) -> f64 {
    match stage {
        1 => 1000.0,
        2 => 1200.0,
        3 => 1500.0,
        4 => 2000.0,
        5 => 0.0,
        _ => 1000.0,
    }
}

fn stage_direction(stage: i32) -> (f64, f64) {
    let mut rng = rand::thread_rng();
    match stage {
        1 | 5 => {
            let angle_deg = rng.gen_range(10.0..20.0);
            let angle_rad = angle_deg * PI / 180.0;
            if rng.gen_bool(0.5) { (angle_rad.cos(), angle_rad.sin()) } else { (-angle_rad.cos(), angle_rad.sin()) }
        }
        2 => {
            let angle_deg = rng.gen_range(10.0..20.0);
            let angle_rad = angle_deg * PI / 180.0;
            if rng.gen_bool(0.5) { (angle_rad.sin(), angle_rad.cos()) } else { (angle_rad.sin(), -angle_rad.cos()) }
        }
        3 => {
            let angle_deg = rng.gen_range(30.0..40.0);
            let angle_rad = angle_deg * PI / 180.0;
            if rng.gen_bool(0.5) { (angle_rad.cos(), angle_rad.sin()) } else { (-angle_rad.cos(), angle_rad.sin()) }
        }
        4 => {
            let angle_deg = rng.gen_range(10.0..20.0);
            let angle_rad = angle_deg * PI / 180.0;
            if rng.gen_bool(0.5) { (angle_rad.cos(), angle_rad.sin()) } else { (-angle_rad.cos(), angle_rad.sin()) }
        }
        _ => (1.0, 1.0),
    }
}

// --- 音频 ---
fn generate_bgm() -> Vec<i16> {
    let duration_secs = 16;
    let num_samples = (SAMPLE_RATE as u64 * duration_secs as u64) as usize;
    let mut out = Vec::with_capacity(num_samples * 2);
    let c4 = 261.63; let e4 = 329.63; let g4 = 392.00; let b4 = 493.88;
    let a3 = 220.00; let f3 = 174.61; let g3 = 196.00; let c3 = 130.81;
    let chords = [vec![c3, c4, e4, g4, b4], vec![a3, c4, e4, g4], vec![f3, c4, e4, 349.23], vec![g3, b4, 293.66, 349.23]];
    
    for i in 0..num_samples {
        let t = i as f64 / SAMPLE_RATE as f64;
        let mut sample = 0.0;
        let chord_idx = ((t / 2.0) as usize) % chords.len();
        let current_chord = &chords[chord_idx];
        let chord_t = t % 2.0;
        
        for &freq in current_chord {
            let mut val = (t * freq * 2.0 * PI).sin() * 0.4;
            val += (t * freq * 6.0 * PI).sin() * 0.15;
            val += (t * freq * 12.0 * PI).sin() * 0.05; 
            let env = (1.0 - chord_t / 2.0).powf(1.5);
            sample += val * env * 0.15;
        }

        let melody_notes = [c4, e4, g4, b4, 440.00, g4, e4, 293.66];
        let mel_idx = ((t / 0.5) as usize) % melody_notes.len();
        let mel_t = t % 0.5;
        let mut mel_val = (t * melody_notes[mel_idx] * 2.0 * PI).sin() * 0.5;
        mel_val += (t * melody_notes[mel_idx] * 4.0 * PI).sin() * 0.2;
        let mel_env = if mel_t < 0.02 { mel_t / 0.02 } else { (1.0 - (mel_t - 0.02) / 0.48).powf(3.0) };
        sample += mel_val * mel_env * 0.12;
        
        if sample > 1.0 { sample = 1.0; } else if sample < -1.0 { sample = -1.0; }
        let v = (sample * 32767.0) as i16;
        out.push(v); out.push(v);
    }
    out
}

fn generate_bounce_sound() -> Vec<i16> {
    let duration = 0.08f64;
    let samples = (SAMPLE_RATE as f64 * duration).round() as usize;
    let mut sound_data = Vec::with_capacity(samples * 2);
    for j in 0..samples {
        let t = j as f64 / SAMPLE_RATE as f64;
        let p = t / duration;
        let freq = 150.0 * (1.0 - p * 0.5);
        let mut s = (2.0 * PI * freq * t).sin() * 0.8 * (-p * 15.0).exp();
        if s > 1.0 { s = 1.0; } else if s < -1.0 { s = -1.0; }
        let v = (s * 32767.0) as i16;
        sound_data.push(v); sound_data.push(v);
    }
    sound_data
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
    let mut event_pump = sdl_context.event_pump().expect("Events Failed");

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
    mut user_config: UserConfig, 
    sink: Sink,
    stream_handle: OutputStreamHandle
) {
    let bounce_data = generate_bounce_sound();
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
                // 使用 Scancode 绕过输入法干扰
                Event::KeyDown { scancode: Some(Scancode::P), .. } => {
                     user_config.audio.bgm_enabled = !user_config.audio.bgm_enabled;
                     // 彻底避免 set_volume，直接通过 sink 内容控制
                     if user_config.audio.bgm_enabled {
                         if sink.empty() {
                             let bgm = generate_bgm();
                             let src = SamplesBuffer::new(2, SAMPLE_RATE, bgm);
                             sink.append(src.repeat_infinite());
                             sink.set_volume(0.5); // 初始化设一次
                         }
                         sink.play();
                     } else {
                         // 彻底暂停或清空
                         sink.pause(); 
                     }
                }
                Event::KeyDown { scancode: Some(Scancode::Space), .. } => {
                     // println!("Space Scancode Pressed");
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
                        if user_config.audio.bgm_enabled && sink.empty() { 
                             let bgm = generate_bgm();
                             let src = SamplesBuffer::new(2, SAMPLE_RATE, bgm);
                             sink.append(src.repeat_infinite()); 
                        // 确保背景音乐播放（强制播放一遍以重置状态）
             if user_config.audio.bgm_enabled {
                 sink.play();
             } 
                        }
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

        // 鼠标光标控制：开始/结束/暂停时显示，游戏中隐藏
        if is_game_over || is_start_screen || paused { 
            sdl_context.mouse().show_cursor(true); 
        } else { 
            sdl_context.mouse().show_cursor(false); 
        }
        
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
                if ball.update(dt) {
                    let bounce_src = SamplesBuffer::new(2, SAMPLE_RATE, bounce_data.clone());
                    let _ = stream_handle.play_raw(bounce_src.convert_samples());
                }
            }
            if stage == 4 && stage_elapsed >= 22.5 && stage_elapsed < 22.6 {
                let mut rng = rand::thread_rng();
                let ang = rng.gen_range(10.0..20.0) * PI / 180.0;
                let dir = if rng.gen_bool(0.5) { (ang.sin(), ang.cos()) } else { (ang.sin(), -ang.cos()) };
                ball.set_speed(stage_speed(stage), Some(dir));
            }
            if (stage_elapsed * 1000.0) as u64 > STAGE_DURATIONS_MS {
                if stage >= 5 { is_game_over = true; sink.stop(); }
                else {
                    stage += 1; is_transitioning = true; transition_timer = 3.0;
                    ball.set_speed(stage_speed(stage), Some(stage_direction(stage)));
                }
            }
        }

        // --- Render ---
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

        let is_fs = match canvas.window().fullscreen_state() { FullscreenType::Off => false, _ => true };
        draw_window_controls(&mut canvas, mouse_pos, is_fs, curr_w as i32);

        let cx = (curr_w / 2) as i32;
        
        if is_game_over {
            let size_go = curr_w as f32 / 8.0;
            let size_restart = curr_w as f32 / 20.0;
            
            // 结束画面文字位置：距离顶部 35%
            let y_go = (curr_h as f32 * 0.35) as i32;
            // 副标题：主标题下方留一定间距
            let y_restart = (y_go as f32 + size_go * 0.8) as i32;

            draw_text(&mut canvas, &font, "GAME OVER", cx, y_go, size_go, Color::RGB(255, 215, 0));
            draw_text(&mut canvas, &font, "PRESS 'R' TO RESTART", cx, y_restart, size_restart, Color::RGB(200, 200, 200));
        } else if is_start_screen {
            let size_title = curr_w as f32 / 8.0;
            let size_start = curr_w as f32 / 20.0;
            let size_footer = curr_w as f32 / 24.0;
            
            // 主标题位置：距离顶部 35%
            let y_title = (curr_h as f32 * 0.35) as i32;
            // 副标题：主标题下方间距减小 (0.8 -> 0.5)
            let y_start = (y_title as f32 + size_title * 0.5) as i32;

            draw_text(&mut canvas, &font, "EYE MOTION", cx, y_title, size_title, Color::RGB(255, 215, 0));
            draw_text(&mut canvas, &font, "PRESS SPACE TO START", cx, y_start, size_start, Color::RGB(255, 105, 120));
             // 底部文字：距离底边为文字高度
             let y_footer = (curr_h as f32 - size_footer) as i32;
             let bgm_txt = if user_config.audio.bgm_enabled { "ON" } else { "OFF" };
             let footer_text = format!("ESC:EXIT - SPACE:START&PAUSE - P:MUSIC ({})", bgm_txt);
             draw_text(&mut canvas, &font, &footer_text, cx, y_footer, size_footer, Color::RGB(135, 206, 235));
        } else if is_transitioning {
            let text = if stage == 5 { "LEVEL R".to_string() } else { format!("LEVEL {}", stage) };
            let size_level = curr_w as f32 / 8.0;
            // LEVEL 和 PAUSED 统一位置：屏幕垂直中心 (0.5 * H)
            let y_center_text = (curr_h as f32 * 0.5) as i32;
            draw_text(&mut canvas, &font, &text, cx, y_center_text, size_level, Color::RGB(255, 215, 0));
        } else {
            ball.draw(&mut canvas);
            let rem = (STAGE_DURATIONS_MS as f64 / 1000.0 - stage_elapsed).max(0.0);
            let txt = format!("TIME: {:.0}", rem);
            let size_time = curr_w as f32 / 30.0;
            draw_text_left(&mut canvas, &font, &txt, cx - (size_time * 1.1) as i32, (curr_h as i32) - 80, size_time, Color::RGB(135, 206, 235));
            if paused {
                 let size_pause = curr_w as f32 / 8.0;
                 // PAUSED 使用相同的位置计算
                 let y_center_text = (curr_h as f32 * 0.5) as i32;
                 draw_text(&mut canvas, &font, "PAUSED", cx, y_center_text, size_pause, Color::RGB(255, 215, 0));
            }
            // 在游戏中左上角显示音乐状态，以便反馈
            let bgm_st = if user_config.audio.bgm_enabled { "ON" } else { "OFF" };
            draw_text_left(&mut canvas, &font, &format!("MUSIC: {}", bgm_st), 10, 30, 24.0, Color::RGB(200, 200, 200));
        }
        
        canvas.present();
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

fn draw_ball_styled<T: sdl2::render::RenderTarget>(
    canvas: &mut sdl2::render::Canvas<T>,
    cx: i32,
    cy: i32,
    radius: f64,
) {
    let r = radius as i32;
    
    // 线性渐变：左上#10B4C3到右下#11C58C
    let color_top_left = (0x10, 0xB4, 0xC3);    // #10B4C3
    let color_bottom_right = (0x11, 0xC5, 0x8C); // #11C58C
    let outline_color = Color::RGB(0x46, 0xE2, 0xD5); // #46E2D5
    
    for dy in -r..=r {
        for dx in -r..=r {
            let dist_sq = (dx * dx + dy * dy) as f64;
            let r_sq = radius * radius;
            
            if dist_sq <= r_sq {
                let dist = dist_sq.sqrt();
                
                // 边缘抗锯齿：最后 1px 做 alpha 衰减
                let mut alpha = 255u8;
                if dist > radius - 1.0 {
                    alpha = ((radius - dist) * 255.0) as u8;
                }

                if dist >= radius - 2.0 {
                    // 描边
                    let c = outline_color;
                    canvas.set_draw_color(Color::RGBA(c.r, c.g, c.b, alpha));
                    let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
                } else {
                    // 内部填充（线性渐变）
                    let gradient_ratio = ((dx + r) as f64 + (dy + r) as f64) / (r as f64 * 4.0);
                    let gradient_ratio = gradient_ratio.max(0.0).min(1.0);
                    
                    let r_val = (color_top_left.0 as f64 + (color_bottom_right.0 as f64 - color_top_left.0 as f64) * gradient_ratio) as u8;
                    let g_val = (color_top_left.1 as f64 + (color_bottom_right.1 as f64 - color_top_left.1 as f64) * gradient_ratio) as u8;
                    let b_val = (color_top_left.2 as f64 + (color_bottom_right.2 as f64 - color_top_left.2 as f64) * gradient_ratio) as u8;
                    
                    canvas.set_draw_color(Color::RGBA(r_val, g_val, b_val, alpha)); // 内部一般 alpha 都是 255，除非非常小的小球
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