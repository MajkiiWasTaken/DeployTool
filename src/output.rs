/************************************************
* File: output.rs
* Author: Michal Švrček
*
* DeployTool colored terminal output
*
* ver. 0.2.0
*************************************************/

use colored::Colorize;

pub fn banner() {
    println!();
    println!("{}", "◆ DeployTool v0.2.0".bright_blue().bold());
    println!("{}", "────────────────────────────────────────".dimmed());
}

pub fn info(message: &str) {
    println!("{} {}", "[INFO]".bright_blue().bold(), message);
}

pub fn step(message: &str) {
    println!("{} {}", "[BUILD]".yellow().bold(), message);
}

pub fn success(message: &str) {
    println!("{} {}", "[ OK ]".green().bold(), message.green());
}

pub fn warning(message: &str) {
    println!("{} {}", "[WARN]".yellow().bold(), message);
}

pub fn error(message: &str) {
    eprintln!("{} {}", "[ERROR]".red().bold(), message.red());
}

pub fn label(name: &str, value: &str) {
    println!("  {:<16} {}", name.bright_black(), value.white());
}

pub fn heading(message: &str) {
    println!();
    println!("{}", message.cyan().bold());
}
