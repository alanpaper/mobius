use crate::markdown::parser::FileMetadata;
use crate::markdown::{parser::FileParser};
use inquire::{Select};
use std::io;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::fs as async_fs;
use tokio::io::AsyncWriteExt;


pub async fn judgement_generate_file_async(file_parser: &FileParser) {
    let generate = Select::new("请选择是否需要生成文件", vec!["是", "否"]).prompt();

    match generate {
        Ok(g) => {
            if g == "是" {
                for file in &file_parser.files {
                    if let Err(e) = generate_file_async(file).await {
                        eprintln!("Error generating file: {}", e);
                    }
                }
            }
        }
        Err(_) => println!("未选择模板"),
    }
}

pub async fn judgement_run_command_async(file_parser: &FileParser) {
    let message = format!("检测到 {} 条命令，是否执行？", file_parser.commands.len());
    let generate = Select::new(&message, vec!["是", "否"]).prompt();

    match generate {
        Ok(g) => {
            if g == "是" {
                for command in &file_parser.commands {
                    if let Err(e) = run_command_async(command).await {
                        eprintln!("Error running command: {}", e);
                    }
                }
            }
        }
        Err(_) => println!("未选择模板"),
    }
}

pub async fn run_command_async(command: &String) -> Result<(), io::Error> {

    let message = format!("检测到 {} 命令，是否执行？", command);
    let generate = Select::new(&message, vec!["是", "否"]).prompt();

    match generate {
        Ok(g) => {
            if g == "是" {
                let command_str = if cfg!(target_os = "windows") {
                    format!("cmd /C {}", &command.as_str())
                } else {
                    command.to_string()
                };

                let output = if cfg!(target_os = "windows") {
                    Command::new("cmd").args(&["/C", &command_str]).output().await
                } else {
                    Command::new("sh").arg("-c").arg(&command_str).output().await
                };

                match output {
                    Ok(output) => {
                        if !output.status.success() {
                            eprintln!("Command failed: {}", command);
                        }
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("Failed to execute command: {}", e);
                        return Err(e);
                    }
                }
            }
        }
        Err(_) => println!("未执行命令"),
    }

    Ok(())


}

pub async fn generate_file_async(file: &FileMetadata) -> Result<(), io::Error> {
    if !file.meta_data.contains_key("path") {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "code block missing path metadata",
        ));
    }
    
    let path = file.meta_data.get("path").unwrap();
    let path = PathBuf::from(&path);

    // Create file if it doesn't exist
    if !path.exists() {
        if let Some(parent) = path.parent() {
            async_fs::create_dir_all(parent).await?;
        }
        async_fs::File::create(&path).await?;
    }

    // Write content to file
    let content = file.content.as_bytes();
    let mut file_handle = async_fs::File::create(&path).await?;
    file_handle.write_all(content).await?;

    println!("File generated successfully at: {:?}", &path);
    Ok(())
}