use std::collections::HashMap;
use std::process::Output;
use colored::Colorize;
use crate::errors::KeeperError;
use error_stack::{Result};

pub mod maven;
pub mod gradle;
pub mod npm;
pub mod cargo;
pub mod sbt;
pub mod composer;
pub mod bundler;
pub mod golang;

pub const COMMANDS: &'static [&'static str] = &["init", "install", "compile", "build", "start", "test", "deps", "doc", "clean", "outdated", "update"];
pub const MANAGERS: &'static [&'static str] = &["maven", "gradle", "sbt", "npm", "cargo", "cmake", "composer", "bundle", "cmake", "go"];

pub fn get_available_managers() -> Vec<String> {
    let mut managers = Vec::new();
    if maven::is_available() {
        managers.push("maven".to_string());
    }
    if gradle::is_available() {
        managers.push("gradle".to_string());
    }
    if sbt::is_available() {
        managers.push("sbt".to_string());
    }
    if npm::is_available() {
        managers.push("npm".to_string());
    }
    if cargo::is_available() {
        managers.push("cargo".to_string());
    }
    if composer::is_available() {
        managers.push("composer".to_string());
    }
    if bundler::is_available() {
        managers.push("bundle".to_string());
    }
    if golang::is_available() {
        managers.push("go".to_string());
    }
    managers
}

pub fn get_manager_file_name(runner: &str) -> &'static str {
    match runner {
        "maven" => "pom.xml",
        "gradle" => "build.gradle[.kts]",
        "sbt" => "build.sbt",
        "npm" => "package.json",
        "cargo" => "Cargo.toml",
        "cmake" => "CMakeLists.txt",
        "composer" => "composer.json",
        "go" => "go.mod",
        "swift" => "Package.swift",
        "bundle" => "Gemfile",
        _ => "unknown",
    }
}

pub fn run_task(runner: &str, task_name: &str, extra_args: &[&str], verbose: bool) -> Result<(), KeeperError> {
    let mut queue: HashMap<&str, fn(&str, &[&str], bool) -> Result<Output, KeeperError>> = HashMap::new();
    if maven::is_available() {
        if maven::is_command_available() {
            queue.insert("maven", maven::run_task);
        } else {
            println!("{}", format!("[tk] maven(https://maven.apache.org/) command not available for pom.xml").bold().red());
        }
    }
    if gradle::is_available() {
        if gradle::is_command_available() {
            queue.insert("gradle", gradle::run_task);
        } else {
            println!("{}", format!("[tk] gradle(https://gradle.org/) command not available").bold().red());
        }
    }
    if sbt::is_available() {
        if sbt::is_command_available() {
            queue.insert("sbt", sbt::run_task);
        } else {
            println!("{}", format!("[tk] sbt(https://www.scala-sbt.org/) command not available for build.sbt").bold().red());
        }
    }
    if npm::is_available() {
        if npm::is_command_available() {
            queue.insert("npm", npm::run_task);
        } else {
            println!("{}", format!("[tk] npm(https://nodejs.org/) command not available for package.json").bold().red());
        }
    }
    if cargo::is_available() {
        if cargo::is_command_available() {
            queue.insert("cargo", cargo::run_task);
        } else {
            println!("{}", format!("[tk] cargo(https://gradle.org/) command not available for Cargo.toml").bold().red());
        }
    }
    if composer::is_available() {
        if composer::is_command_available() {
            queue.insert("composer", composer::run_task);
        } else {
            println!("{}", format!("[tk] gradle(https://gradle.org/) command not available for composer.json").bold().red());
        }
    }
    if bundler::is_available() {
        if bundler::is_command_available() {
            queue.insert("bundle", bundler::run_task);
        } else {
            println!("{}", format!("[tk] bundle(https://bundler.io/) command not available for Gemfile").bold().red());
        }
    }
    if golang::is_available() {
        if golang::is_command_available() {
            queue.insert("go", golang::run_task);
        } else {
            println!("{}", format!("[tk] go(https://go.dev/) command not available for go.mod").bold().red());
        }
    }
    if queue.is_empty() { // no manager found
        println!("{}", format!("[tk] no available manager detected").bold().red());
    } else if !runner.is_empty() { // run task by runner name
        if let Some(task) = queue.get(runner) {
            println!("{}", format!("[tk] execute {} from {}", task_name, runner).bold().blue());
            task(task_name, extra_args, verbose).unwrap();
        } else {
            println!("{}", format!("[tk] {} manager not available", runner).bold().red());
        }
    } else { // run task by all available managers
        match task_name {
            "init" => {}
            "start" => { // only execute start task once
                if queue.len() == 1 {
                    queue.iter().for_each(|(runner_name, task)| {
                        println!("{}", format!("[tk] execute {} from {}", task_name, runner_name).bold().blue());
                        task(task_name, extra_args, verbose).unwrap();
                    });
                } else {
                    let runner_names = queue.iter().map(|(runner_name, _task)| runner_name.to_owned()).collect::<Vec<_>>().join(",");
                    println!("{}", format!("[tk] Failed to run start because of multi start tasks from {}", runner_names).bold().red());
                }
            }
            _ => {
                queue.iter().for_each(|(runner_name, task)| {
                    println!("{}", format!("[tk] execute {} from {}", task_name, runner_name).bold().blue());
                    task(task_name, extra_args, verbose).unwrap();
                });
            }
        }
    }
    Ok(())
}
