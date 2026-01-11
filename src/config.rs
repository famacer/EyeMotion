use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 用户配置结构体 - 用于持久化保存用户设置和训练记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// 音频设置
    pub audio: AudioConfig,
    
    /// 训练记录
    pub training_stats: TrainingStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// 背景音乐是否启用
    pub bgm_enabled: bool,
    
    /// 背景音乐音量 (0.0 - 1.0)
    pub bgm_volume: f32,
    
    /// 音效音量 (0.0 - 1.0)
    pub sfx_volume: f32,
    
    /// 当前选择的碰撞音效索引 (0-9)
    pub selected_bounce_sound: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingStats {
    /// 完成的训练会话数
    pub sessions_completed: u32,
    
    /// 总训练时间（秒）
    pub total_training_time: f64,
    
    /// 达到的最高阶段
    pub highest_stage_reached: i32,
}

impl Default for UserConfig {
    fn default() -> Self {
        UserConfig {
            audio: AudioConfig {
                bgm_enabled: true,
                bgm_volume: 0.15,
                sfx_volume: 1.0,
                selected_bounce_sound: 0,
            },
            training_stats: TrainingStats {
                sessions_completed: 0,
                total_training_time: 0.0,
                highest_stage_reached: 0,
            },
        }
    }
}

impl UserConfig {
    /// 获取配置文件路径
    /// Windows: %APPDATA%/eyemotion/config.toml
    fn get_config_path() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let mut path = PathBuf::from(appdata);
                path.push("eyemotion");
                return Some(path);
            }
        }
        
        // 如果获取失败，使用当前目录
        Some(PathBuf::from("."))
    }
    
    /// 从配置文件加载配置，如果文件不存在则返回默认配置
    pub fn load() -> Self {
        if let Some(config_dir) = Self::get_config_path() {
            let config_file = config_dir.join("config.toml");
            
            if config_file.exists() {
                if let Ok(content) = fs::read_to_string(&config_file) {
                    if let Ok(config) = toml::from_str::<UserConfig>(&content) {
                        return config;
                    }
                }
            }
        }
        
        // 加载失败时返回默认配置
        UserConfig::default()
    }
    
    /// 保存配置到文件
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_dir) = Self::get_config_path() {
            // 确保目录存在
            fs::create_dir_all(&config_dir)?;
            
            let config_file = config_dir.join("config.toml");
            let toml_string = toml::to_string_pretty(self)?;
            fs::write(config_file, toml_string)?;
        }
        Ok(())
    }
    
    /// 更新训练统计信息
    pub fn update_training_stats(&mut self, stage_completed: i32, time_spent: f64) {
        self.training_stats.sessions_completed += 1;
        self.training_stats.total_training_time += time_spent;
        
        if stage_completed > self.training_stats.highest_stage_reached {
            self.training_stats.highest_stage_reached = stage_completed;
        }
    }
}
