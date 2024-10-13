use std::io::{BufRead, BufReader, Stdin, Write};
use std::path::{Path, PathBuf};
use std::process::{Stdio};
use clap::{Command, Arg, ArgAction, ArgMatches};
use colored::Colorize;
use just::summary::Summary;

pub const VERSION: &str = "0.1.0";

const SUB_COMMANDS: [&str; 3] = ["list", "add", "edit"];

fn main() {
    let args = Vec::from_iter(std::env::args());
    if args.len() >= 2 && !SUB_COMMANDS.contains(&args[1].as_str()) {
        let args = args.iter().map(|v| v.as_ref()).collect::<Vec<_>>();
        run_snippet(&args);
        return;
    }
    let app = build_sq_app();
    let matches = app.get_matches();
    match matches.subcommand() {
        Some(("list", _)) => {
            list_snippets();
        }
        Some(("add", _)) => {
            add_snippet();
        }
        Some(("edit", edit_matches)) => {
            edit_snippet(edit_matches);
        }
        _ => {
            println!("Unknown command");
        }
    }
}

fn get_snippets_file() -> PathBuf {
    let tk_home = dirs::home_dir().unwrap().join(".tk");
    let snippets_file_path = tk_home.join("snippets.just");
    if !snippets_file_path.exists() {
        if !tk_home.exists() {
            std::fs::create_dir_all(tk_home).unwrap();
        }
        std::fs::write(&snippets_file_path, include_bytes!("templates/just/snippets.just")).unwrap();
    }
    snippets_file_path
}

pub fn run_snippet(args: &[&str]) {
    let snippet_file = get_snippets_file();
    let mut just_args = vec!["just", "-f", snippet_file.to_str().unwrap()];
    just_args.extend(args.iter().skip(1));
    std::env::set_var("JUST_UNSTABLE", "1");
    if let Err(code) = just::run(just_args.iter()) {
        std::process::exit(code);
    }
}

fn list_snippets() {
    let snippet_file = get_snippets_file();
    let just_args = vec!["just", "-f", snippet_file.to_str().unwrap(), "--list"];
    if let Err(code) = just::run(just_args.iter()) {
        std::process::exit(code);
    }
}

fn add_snippet() {
    let snippets_file_path = get_snippets_file();
    let code = std::fs::read_to_string(&snippets_file_path).unwrap();
    let snippets_file_path = get_snippets_file();
    let summary = just::summary::summary(&snippets_file_path).unwrap().unwrap();
    let mut cli = String::new();
    let mut name = String::new();
    let mut description = String::new();
    // read cli
    print!("{}", "Cli: ".bold());
    std::io::stdout().flush().unwrap();
    let stdin = std::io::stdin();
    stdin.read_line(&mut cli).unwrap();
    // read name
    name = read_snippet_name(&stdin, &summary);
    // read description
    print!("{}", "Description: ".bold());
    std::io::stdout().flush().unwrap();
    stdin.read_line(&mut description).unwrap();
    // append to snippets file
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(&snippets_file_path)
        .unwrap();
    file.write(format!("\n# {}\n{}:\n  {}\n", description.trim(), name.trim(), cli).as_bytes()).unwrap();
    println!("{} added successfully", name.trim());
}

fn read_snippet_name(stdin: &Stdin, summary: &Summary) -> String {
    let mut name = String::new();
    print!("{}", "Name: ".bold());
    std::io::stdout().flush().unwrap();
    stdin.read_line(&mut name).unwrap();
    let temp_name = name.trim();
    if summary.recipes.contains_key(temp_name) {
        println!("{}", format!("Snippet of {} exits already, please input another name", temp_name).red());
        return read_snippet_name(stdin, summary);
    }
    name
}

fn edit_snippet(matches: &ArgMatches) {
    let editor_name = if matches.get_flag("vscode") {
        "code"
    } else if matches.get_flag("zed") {
        "zed"
    } else {
        &std::env::var("EDITOR").unwrap_or_else(|_|
            // default to vim or vi
            if which::which("vim").is_ok() {
                "vim".to_owned()
            } else {
                "vi".to_owned()
            }
        )
    };
    let snippet_file_path = get_snippets_file();
    let snippets_file = snippet_file_path.to_str().unwrap();
    // line number to navigate
    let line_number = if matches.get_flag("end") {
        count_lines(&snippet_file_path).unwrap()
    } else if let Some(name) = matches.get_one::<String>("name") {
        get_recipe_line_number(name)
    } else {
        0
    };
    if line_number > 0 {
        if editor_name == "code" { // open with code
            let location = format!("{}:{}", snippets_file, line_number);
            run_command("code", &["--goto", &location]);
        } else if editor_name == "zed" { // open with zed
            let location = format!("{}:{}", snippets_file, line_number);
            run_command("zed", &[&location]);
        } else if editor_name.starts_with("vi") { // open with vi
            let location = format!("+{}", line_number);
            run_command(editor_name, &[&location, snippets_file]);
        } else {
            run_command(editor_name, &[snippets_file]);
        }
    } else {
        run_command(editor_name, &[snippets_file]);
    }
}

fn run_command(command_name: &str, args: &[&str]) {
    let mut command = std::process::Command::new(command_name);
    command
        .args(args)
        .envs(std::env::vars())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
}

pub fn get_recipe_line_number(name: &str) -> usize {
    let snippet_file = get_snippets_file();
    let file = std::fs::File::open(snippet_file).unwrap();
    let reader = BufReader::new(file);
    let mut line_number = 0;
    for line in reader.lines() {
        line_number += 1;
        if let Ok(line) = line {
            if line.starts_with(name) {
                return line_number;
            }
        }
    }
    line_number + 1
}

pub fn count_lines<P: AsRef<Path>>(file_path: P) -> Result<usize, std::io::Error> {
    let handle = std::fs::File::open(file_path)?;
    let mut reader = BufReader::with_capacity(1024 * 32, handle);
    let mut count = 0;
    loop {
        let len = {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break;
            }
            count += bytecount::count(&buf, b'\n');
            buf.len()
        };
        reader.consume(len);
    }
    Ok(count)
}

pub fn build_sq_app() -> Command {
    Command::new("sq")
        .version(VERSION)
        .about("Command-line snippets keeper")
        .subcommand(
            Command::new("list")
                .about("List cli snippets")
        )
        .subcommand(
            Command::new("add")
                .about("Add a new snippet")
        )
        .subcommand(
            Command::new("edit")
                .about("Edit cli snippet")
                .arg(
                    Arg::new("vscode")
                        .long("vscode")
                        .help("Open VS Code to edit snippet")
                        .num_args(0)
                        .action(ArgAction::SetTrue)
                        .required(false),
                )
                .arg(
                    Arg::new("zed")
                        .long("zed")
                        .help("Open Zed to edit snippet")
                        .num_args(0)
                        .action(ArgAction::SetTrue)
                        .required(false),
                )
                .arg(
                    Arg::new("end")
                        .long("end")
                        .help("Open editor and navigate to end and edit snippet")
                        .num_args(0)
                        .action(ArgAction::SetTrue)
                        .required(false),
                )
                .arg(
                    Arg::new("name")
                        .help("Open editor to edit snippet")
                        .num_args(1)
                        .index(1)
                )
        )
}

#[cfg(test)]
mod tests {
    use just::summary::summary;
    use super::*;

    #[test]
    fn test_list_snippets() {
        list_snippets();
    }

    #[test]
    fn test_count_lines() {
        let lines: usize = count_lines("Cargo.toml").unwrap();
        println!("lines: {}", lines);
    }

    #[test]
    fn test_get_recipe_line_number() {
        let line_number = get_recipe_line_number("public-ip");
        println!("line_number: {}", line_number);
    }
}
