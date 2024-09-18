use clap::{Parser, Subcommand, Args};

/// Utility to add executables, binaries, AppDirs, AppImages as desktop files without user
/// intervention
#[derive(Parser, Debug)]
#[command(name = "appreg", author, version, about)]
pub struct Cli {
    // Only print what would be done do not install any desktop files
    #[arg(long)]
    pub dry_run: bool,

    /// Extend app directories, defaults to ~/Apps and ~/.apps
    #[arg(short, long)]
    pub app_dir: Vec<String>,

    #[command(subcommand)]
    pub cmd: CliCommands,
}

#[derive(Subcommand, Debug)]
pub enum CliCommands {
    /// Updates all desktop files created by appreg, removes broken ones
    Update,

    /// Install desktop file for specific app
    Install(CmdInstallArgs),
}

#[derive(Args, Debug)]
pub struct CmdInstallArgs {
    /// File to create desktop file for, can be an AppDir, AppImage, binary or executable file
    pub file: String,
}

