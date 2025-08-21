use clap::{
    ColorChoice, Parser, Subcommand,
    builder::{Styles, styling::AnsiColor},
};

#[derive(Parser)]
#[command(disable_version_flag = true)]
#[command(disable_help_flag = true)]
#[command(color = ColorChoice::Auto)]
#[command(styles = get_styles())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(name = "init", about = "init migration folder and changelog table")]
    Init,
    #[command(name = "create", about = "create new migration")]
    Create {
        #[arg(value_name = "NAME", help = "name of migration")]
        name: String,
    },
    #[command(name = "up", about = "run all unapplied database migrations")]
    Up,
    #[command(name = "down", about = "undo the last applied database migrations")]
    Down,
    #[command(name = "status", about = "print the changelog of the database")]
    Status,
    #[command(name = "drop", about = "remove last pending migration")]
    Drop,
    #[command(name = "info", about = "get info about application")]
    Info,
    #[command(name = "version", about = "get version of application")]
    Version,
}

fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default())
        .usage(AnsiColor::Green.on_default())
        .literal(AnsiColor::Blue.on_default().bold())
        .error(AnsiColor::Red.on_default())
        .valid(AnsiColor::Green.on_default())
        .invalid(AnsiColor::Red.on_default())
}
