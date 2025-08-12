use clap::ValueEnum;
use serde::{Deserialize, Serialize};

// 主题枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum Theme {
    Light,
    Dark,
    System,
}
