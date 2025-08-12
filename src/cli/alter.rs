use clap::Parser;
use std::error::Error;
use std::fs::{self, File};
use std::path::PathBuf;

use crate::session::main_loop::main_loop;
use crate::session::manager::{SessionManager};

use crate::cli::actions::{Commands, ConfigSubcommand, McpSubcommand};

#[derive(Parser)]
#[command(name = "会话管理")]
#[command(version = "1.0")]
#[command(about = "一个会话管理系统", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

pub struct Alter;

impl Alter {
    pub fn new() -> Self {
        Alter
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("session_manager");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let config_path = config_dir.join("config.json");
        let sessions_path = config_dir.join("sessions.json");

        let mut session_manager = SessionManager::new(config_path)?;

        if let Err(e) = session_manager.load_sessions(&sessions_path) {
            eprintln!("警告: 无法加载会话数据: {}", e);
        }

        let cli = Cli::parse();

        match cli.command {
            Commands::Start { title, restore } => {
                if let Some(session_id) = restore {
                    if session_manager.switch_session(&session_id).is_ok() {
                        println!("已恢复会话: {}", session_id);
                    } else {
                        println!("会话未找到: {}", session_id);
                        let new_id =
                            session_manager.create_session(title.as_deref().unwrap_or("新会话"));
                        println!("已创建新会话: {}", new_id);
                    }
                } else if session_manager.current_session_id.is_none() {
                    let new_id =
                        session_manager.create_session(title.as_deref().unwrap_or("新会话"));
                    println!("已创建新会话: {}", new_id);
                }

                main_loop(&mut session_manager, &sessions_path).await?;
            }

            Commands::Config { subcommand } => match subcommand {
                ConfigSubcommand::Show => {
                    println!("当前配置:");
                    println!("  最大会话数: {}", session_manager.config.max_sessions);
                    println!("  自动保存: {}", session_manager.config.auto_save);
                    println!("  默认模型: {}", session_manager.config.default_model.name.unwrap());
                    println!("  主题: {:?}", session_manager.config.theme);
                }

                ConfigSubcommand::SetMaxSessions { max } => {
                    session_manager.config.max_sessions = max;
                    session_manager.save_config()?;
                    println!("最大会话数已设置为: {}", max);
                }

                ConfigSubcommand::SetModel { model } => {
                    session_manager.config.default_model.name = Some(model);
                    session_manager.save_config()?;
                    println!("默认模型已设置为: {:#?}", session_manager.config.default_model);
                }

                ConfigSubcommand::SetTheme { theme } => {
                    session_manager.config.theme = theme;
                    session_manager.save_config()?;
                    println!("主题已设置为: {:?}", theme);
                }

                ConfigSubcommand::ToggleAutoSave => {
                    session_manager.config.auto_save = !session_manager.config.auto_save;
                    session_manager.save_config()?;
                    println!(
                        "自动保存已{}",
                        if session_manager.config.auto_save {
                            "启用"
                        } else {
                            "禁用"
                        }
                    );
                }
            },

            Commands::Generate { session_id } => match session_manager.switch_session(&session_id) {
                Ok(_) => {
                    println!("确定要生成改对话吗?: {}", session_id);
                    let _ = session_manager.generate_session_file(Some(&session_id)).await;
                }
                Err(e) => eprintln!("错误: {}", e),
            },

            Commands::Mcp { subcommand } => {
                match subcommand {
                    McpSubcommand::Rename {
                        session_id,
                        new_title,
                    } => match session_manager.rename_session(&session_id, &new_title) {
                        Ok(_) => println!("会话 '{}' 已重命名为 '{}'", session_id, new_title),
                        Err(e) => eprintln!("错误: {}", e),
                    },

                    McpSubcommand::Delete { session_id } => {
                        match session_manager.remove_session(&session_id) {
                            Ok(_) => println!("会话 '{}' 已删除", session_id),
                            Err(e) => eprintln!("错误: {}", e),
                        }
                    }

                    McpSubcommand::Cleanup => {
                        let old_count = session_manager.sessions.len();
                        session_manager.cleanup_old_sessions();
                        let new_count = session_manager.sessions.len();
                        println!(
                            "已清理 {} 个会话, 剩余 {} 个会话",
                            old_count - new_count,
                            new_count
                        );
                    }
                }

                // 保存会话
                session_manager.save_sessions(&sessions_path)?;
            }

            Commands::List { detail, all } => {
                let sessions = session_manager.list_sessions();

                if sessions.is_empty() {
                    println!("没有可用的会话");
                    return Ok(());
                }

                println!("{}会话:", if all { "所有 " } else { "" });
                println!();

                for (i, session) in sessions.iter().enumerate() {
                    let current_indicator =
                        if Some(&session.id) == session_manager.current_session_id.as_ref() {
                            " (当前)"
                        } else {
                            ""
                        };

                    if detail {
                        println!(
                            "{}. {} [ID: {}]{}",
                            i + 1,
                            session.title,
                            session.id,
                            current_indicator
                        );
                        println!(
                            "  创建时间: {}",
                            session.created_at.format("%Y-%m-%d %H:%M:%S")
                        );
                        println!(
                            "  最后访问: {}",
                            session.last_accessed.format("%Y-%m-%d %H:%M:%S")
                        );
                        println!("  消息数量: {}", session.messages.len());
                    } else {
                        println!(
                            "{}. {} [ID: {}]{}",
                            i + 1,
                            session.title,
                            &session.id[..8],
                            current_indicator
                        );
                    }

                    if !detail {
                        println!();
                    }
                }
            }

            Commands::Resume => {
                if let Some(session_id) = &session_manager.current_session_id {
                    println!("正在恢复上一次会话: {}", session_id);
                    main_loop(&mut session_manager, &sessions_path).await?;
                } else {
                    println!("没有可恢复的上一次会话");
                    println!("正在创建新会话...");
                    let _ = session_manager.create_session("新会话");
                    main_loop(&mut session_manager, &sessions_path).await?;
                }
            }

            Commands::Restore { session_id } => match session_manager.switch_session(&session_id) {
                Ok(_) => {
                    println!("已切换到会话: {}", session_id);
                    main_loop(&mut session_manager, &sessions_path).await?;
                }
                Err(e) => eprintln!("错误: {}", e),
            },

            Commands::Export {
                path,
                all,
                session_id,
            } => {
                if all {
                    let session_file = File::create(&path)?;
                    serde_json::to_writer_pretty(session_file, &session_manager.sessions)?;
                    println!(
                        "已导出 {} 个会话到 {}",
                        session_manager.sessions.len(),
                        path.display()
                    );
                } else if let Some(id) = session_id {
                    if let Some(session) = session_manager.sessions.get(&id) {
                        let session_file = File::create(&path)?;
                        serde_json::to_writer_pretty(session_file, session)?;
                        println!("已导出会话 '{}' 到 {}", id, path.display());
                    } else {
                        eprintln!("错误: 未找到会话 {}", id);
                    }
                } else {
                    if let Some(current) = session_manager.current_session_id.as_ref() {
                        if let Some(session) = session_manager.sessions.get(current) {
                            let session_file = File::create(&path)?;
                            serde_json::to_writer_pretty(session_file, session)?;
                            println!("已导出当前会话到 {}", path.display());
                        }
                    } else {
                        eprintln!("错误: 没有当前会话");
                    }
                }
            }

            Commands::Import { path } => {
                session_manager.load_sessions(&path)?;
                println!("已从 {} 导入会话", path.display());

                // 保存到默认位置
                session_manager.save_sessions(&sessions_path)?;
            }
        }

        Ok(())
    }
}

pub fn handle_command(
    command: &str,
    session_manager: &mut SessionManager,
    sessions_path: &PathBuf,
) -> Result<bool, Box<dyn Error>> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(false);
    }

    match parts[0] {
        "exit" => {
            println!("已退出，会话已保存");
            return Ok(true);
        }

        "list" => {
            println!("\n--- 当前会话列表 ---");
            for (i, session) in session_manager.list_sessions().iter().enumerate() {
                let current_indicator =
                    if Some(&session.id) == session_manager.current_session_id.as_ref() {
                        " (当前)"
                    } else {
                        ""
                    };
                println!(
                    "{}. {} [ID: {}]{}",
                    i + 1,
                    session.title,
                    &session.id[..8],
                    current_indicator
                );
            }
        }

        "switch" if parts.len() > 1 => {
            let session_id = parts[1];
            match session_manager.switch_session(session_id) {
                Ok(_) => {
                    println!("已切换到会话: {}", session_id);
                    // 更新显示
                    if let Some(session) = session_manager.sessions.get(session_id) {
                        println!("当前会话: {} [ID: {}]", session.title, &session_id[..8]);
                    }
                }
                Err(e) => println!("错误: {}", e),
            }
        }

        "new" => {
            let title = parts.get(1).map(|s| *s).unwrap_or("新会话");
            let new_id = session_manager.create_session(title);
            println!("已创建新会话: {}", new_id);
            println!("当前会话: {} [ID: {}]", title, &new_id[..8]);
        }

        "generate" => {
            let session_id = parts.get(1).map(|s| *s);
            let _ = session_manager.generate_session_file(session_id);
            println!("对话文件已生成: {:?}", session_id);
        }

        "save" => {
            session_manager.save_sessions(sessions_path)?;
            println!("会话已手动保存");
        }

        "rename" if parts.len() > 1 => {
            let new_title = parts[1..].join(" ");
            if let Some(session) = session_manager.get_current_session() {
                session.update_title(&new_title);
                println!("当前会话已重命名为: {}", new_title);
            }
        }

        "title" => {
            if let Some(session) = session_manager.get_current_session() {
                println!("当前会话标题: {}", session.title);
            }
        }

        "config" => {
            println!("当前配置:");
            println!("  最大会话数: {}", session_manager.config.max_sessions);
            println!("  自动保存: {}", session_manager.config.auto_save);
            println!("  默认模型: {:?}", session_manager.config.default_model);
            println!("  主题: {:?}", session_manager.config.theme);
        }

        "help" => {
            print_help();
        }

        _ => {
            println!("未知命令: {}", command);
            print_help();
        }
    }

    Ok(false)
}

fn print_help() {
    println!("\n可用命令:");
    println!("  /exit             - 退出");
    println!("  /list             - 列出所有会话");
    println!("  /switch <ID>      - 切换到指定会话");
    println!("  /new [标题]       - 创建新会话");
    println!("  /save             - 手动保存会话");
    println!("  /rename <新标题>  - 重命名当前会话");
    println!("  /title            - 显示当前会话标题");
    println!("  /config           - 显示当前配置");
    println!("  /help             - 显示帮助");
}
