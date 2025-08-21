use colored::Colorize;

use crate::config;
use momitroll_util::common::get_app_version;

pub const LOGO: [&str; 7] = [
    r"___ ___   ___   ___ ___  ____  ______  ____   ___   *      *     ",
    r"|   |   | /   \ |   |   ||    ||      ||    \ /   \ | |    | |    ",
    r"| *   * ||     || *   * | |  | |      ||  D  )     || |    | |    ",
    r"|  \_/  ||  O  ||  \_/  | |  | |_|  |_||    /|  O  || |___ | |___ ",
    r"|   |   ||     ||   |   | |  |   |  |  |    \|     ||     ||     |",
    r"|   |   ||     ||   |   | |  |   |  |  |  .  \     ||     ||     |",
    r"|___|___| \___/ |___|___||____|  |__|  |__|\_|\___/ |_____||_____|",
];

pub fn print_info() {
    for line in LOGO {
        println!("{}", line.green());
    }
    println!(
        "Repository: {}\nv. {}",
        "https://github.com/gaussfff/momitroll".magenta(),
        get_app_version().red()
    );
}

pub fn print_version() {
    println!(
        "{} {}{}",
        config::APP_NAME.blue(),
        "v.".blue(),
        get_app_version().red()
    );
}
