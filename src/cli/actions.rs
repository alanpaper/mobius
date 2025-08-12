
use clap::Subcommand;
use std::path::PathBuf;

use crate::session::theme::Theme;

#[derive(Subcommand)]
pub enum Commands {
    Start {
        #[arg(short, long)]
        title: Option<String>,
        
        #[arg(short, long)]
        restore: Option<String>,
    },
    
    Config {
        #[command(subcommand)]
        subcommand: ConfigSubcommand,
    },
    
    Mcp {
        #[command(subcommand)]
        subcommand: McpSubcommand,
    },
    
    List {
        #[arg(short, long)]
        detail: bool,
        
        #[arg(short, long)]
        all: bool,
    },
    
    Resume,
    
    Restore {
        session_id: String,
    },

    Generate {
        session_id: String,
    },
    
    Export {
        path: PathBuf,
        
        #[arg(short, long)]
        all: bool,
        
        #[arg(short, long)]
        session_id: Option<String>,
    },
    
    Import {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum ConfigSubcommand {
    Show,
    
    SetMaxSessions {
        max: usize,
    },
    
    SetModel {
        model: String,
    },
    
    SetTheme {
        theme: Theme,
    },
    
    ToggleAutoSave,
}

#[derive(Subcommand)]
pub enum McpSubcommand {
    Rename {
        session_id: String,
        new_title: String,
    },
    
    Delete {
        session_id: String,
    },
    
    Cleanup,
}
