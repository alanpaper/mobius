use std::{collections::HashMap};
use std::result::Result::Ok;
use regex::Regex;
use crate::markdown::generate::{judgement_generate_file_async, judgement_run_command_async};
use anyhow::{Context, Result};

#[derive(Debug)]
pub struct FileMetadata {
    pub meta_data: HashMap<String, String>,
    pub content: String,
    // pub action: Action,
    // pub code_type: String,
}

/// ### 1. 首页组件 `src/pages/Home.tsx`
/// ```typescript
/// <!-- FILE_METADATA
/// path: src/pages/Home.tsx
/// action: create
/// -->
/// import { ArrowRightIcon } from '@heroicons/react/24/outline'

/// export default function Home() {
///   return (
///     <div className="min-h-screen bg-gray-50">
///       {/* 导航栏代码同前... */}
///     </div>
///   )
/// }
/// ```

/// ### 2. 路由配置 `src/App.tsx`
/// ```typescript
/// <!-- FILE_METADATA
/// path: src/App.tsx
/// action: modify
/// -->
/// import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
/// import Home from './pages/Home'

/// function App() {
///   return (
///     <Router>
///       <Routes>
///         <Route path="/" element={<Home />} />
///       </Routes>
///     </Router>
///   )
/// }

/// export default App
/// ```

/// ### 3. 路由安装命令
/// ```bash
/// <!-- FILE_METADATA
/// path: (CLI Command)
/// action: execute
/// -->
/// npm install react-router-dom @types/react-router-dom
/// ```

/// ---

/// ### 调整说明：
/// 1. **严格遵循方案1**：所有文件均包含YAML元数据块
/// 2. **元数据位置**：固定在文件起始位置，格式统一为注释包裹
/// 3. **内容隔离**：元数据与实际代码通过注释分隔线明确区分
/// 4. **命令标注**：CLI命令也使用相同元数据格式标注
#[derive(Debug)]
pub struct FileParser {
    pub files: Vec<FileMetadata>,
    pub commands: Vec<String>,
}

impl FileParser {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            commands: Vec::new(),
        }
    }

    pub async fn init(&mut self, content: String) -> Result<()> {
        let mut markdown = FileParser::new();
        markdown.parse_file(content)
            .context("Failed to parse markdown content")?;

        judgement_generate_file_async(&markdown).await;
        judgement_run_command_async(&markdown).await;

        println!("文件解析完成，生成文件数量: {:#?}", markdown);
        Ok(())
    }

    pub fn parse_file(&mut self, content: String) -> Result<()> {
        self.split_code(content)
            .context("Failed to split code blocks")
    }

    fn split_code(&mut self, content: String) -> Result<()> {
        println!("开始解析文件内容...");
        let re = Regex::new(r"```[\s\S]*?```")
            .context("Failed to compile code block regex")?;
        let code_blocks = re.captures_iter(&content);

        for captures in code_blocks {
            let full_match = captures.get(0)
                .context("Failed to get regex match")?
                .as_str();
            
            // Check if we have enough characters to remove ```
            if full_match.len() < 6 {
                eprintln!("Warning: Found code block that's too short: {}", full_match);
                continue;
            }
            
            let code_block = &full_match[3..full_match.len()-3].trim();
            
            let meta_data = match self.get_file_metadata(code_block.to_string()) {
                Ok(meta) => meta,
                Err(e) => {
                    eprintln!("Warning: Failed to parse metadata for code block: {}", e);
                    HashMap::new()
                }
            };

            let mut code = String::new();
            if let Some(action) = meta_data.get("action") {
                if action == "execute" {
                    if let Err(e) = self.get_file_commands(code_block.to_string()) {
                        eprintln!("Warning: Failed to parse commands: {}", e);
                    }
                } else {
                    match self.get_file_content(code_block.to_string()) {
                        Ok(c) => code = c,
                        Err(e) => eprintln!("Warning: Failed to extract file content: {}", e)
                    }
                }
            } else {
                // Default behavior - treat as file content
                match self.get_file_content(code_block.to_string()) {
                    Ok(c) => code = c,
                    Err(e) => eprintln!("Warning: Failed to extract file content: {}", e)
                }
            }

            self.files.push(FileMetadata {
                meta_data,
                content: code,
            });
        }
        Ok(())
    }

    fn get_file_metadata(&self, code: String) -> Result<HashMap<String, String>> { 
        let re = Regex::new(r"<!-- FILE_METADATA\n([\s\S]*?)\n-->")
            .context("Failed to compile metadata regex")?;
        
        let captures = re.captures(&code)
            .context("No metadata block found in code block")?;
        
        let metadata_str = captures.get(1)
            .context("Metadata section not found")?
            .as_str();
        
        let metadata: HashMap<String, String> = metadata_str
            .split('\n')
            .filter_map(|line| {
                if line.trim().is_empty() {
                    return None;
                }
                
                let parts: Vec<&str> = line.split(": ").collect();
                if parts.len() != 2 {
                    eprintln!("Warning: Invalid metadata line format: {}", line);
                    return None;
                }
                Some((parts[0].to_string(), parts[1].to_string()))
            })
            .collect();
            
        Ok(metadata)
    }

    fn get_file_content(&self, code: String) -> Result<String> {
        let lines: Vec<&str> = code.lines().collect();
        let end_metadata_idx = lines
            .iter()
            .position(|line| line.trim() == "-->")
            .map(|i| i + 1)
            .unwrap_or(0);

        if end_metadata_idx >= lines.len() {
            return Ok(String::new());
        }

        let content = lines[end_metadata_idx..]
            .iter()
            .skip_while(|line| line.trim().is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join("\n");

        Ok(content)
    }

    fn get_file_commands(&mut self, code: String) -> Result<Vec<String>> {
        let lines: Vec<&str> = code.lines().collect();
        let end_metadata_idx = lines
            .iter()
            .position(|line| line.trim() == "-->")
            .map(|i| i + 1)
            .unwrap_or(0);

        if end_metadata_idx >= lines.len() {
            return Ok(vec![]);
        }

        let content = lines[end_metadata_idx..]
            .iter()
            .filter(|&f| {
                // Skip comments (lines starting with #)
                if f.trim_start().starts_with("#") {
                    false
                } else if !f.trim().is_empty() {
                    self.commands.push(f.to_string());
                    true
                } else {
                    false
                }
            })
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        Ok(content)
    }
}