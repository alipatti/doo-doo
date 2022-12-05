#![warn(clippy::pedantic)]

use clap::Parser as _;
use dialoguer::theme::ColorfulTheme;
use std::{error::Error, fs, process::Command};

mod cli;
mod io;
mod structs;

// horizontal bar, not a dash (-)
const LINE_CHAR: char = '─';

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();
    let cols = io::terminal_size().cols as usize;

    let todo_file_path = {
        let mut path = if args.global {
            dirs::home_dir().expect("Home directory not defined.")
        } else {
            std::env::current_dir().expect("Unable to get current directory.")
        };
        path.push("todo.yaml");
        path.to_str().unwrap().to_owned()
    };

    // load and filter tasks based on command line parameters
    let mut tasks = io::read_yaml(&todo_file_path);

    if args.completed {
        // TODO implement completed flag
        // somehow, we need to put the completed tasks aside and add them back after the user interaction.
        // if we just remove all the complete tasks, then
        unimplemented!()
    }

    if args.include_code {
        // TODO implement code flag
        // maybe using the grep library? (used by ripgrep)
        unimplemented!()
    }

    match args.sort.unwrap_or_default() {
        // use (y, x) as closure parameters to reverse sort order (most recent first)
        cli::SortOptions::Date => tasks.sort_by(|y, x| x.due.cmp(&y.due)),
        cli::SortOptions::Title => tasks.sort_by(|y, x| x.title.cmp(&y.title)),
    }

    let tasks_as_strings: Vec<String> = tasks.iter().map(|task| task.to_string(cols)).collect();

    if let Some(command) = args.command {
        match command {
            cli::Commands::List => io::print_tasks(&tasks, cols),

            cli::Commands::Open => {
                Command::new("open")
                    .arg(&todo_file_path)
                    .spawn()
                    .expect("Failed to open file.");
            }

            cli::Commands::Add {
                title,
                details,
                due,
            } => tasks.push(structs::Task {
                title,
                details,
                due: due.map(|duration| {
                    let now = chrono::Local::now();
                    // HACK convert humantime::Duration -> std::time::Duration -> chrono::Duration
                    let delta = chrono::Duration::from_std(duration.into()).unwrap();
                    now + delta
                }),
                completed: false,
            }),

            cli::Commands::Done { task } => {
                if let Some(_task) = task {
                    unimplemented!();
                    // filter tasks based on item
                    // if there's one thing left, remove it.
                    // if there are multiple matches, prompt user to select from them
                } else {
                    io::prompt_user(
                        &tasks_as_strings,
                        // remove gray check mark for unselected items
                        Some(ColorfulTheme {
                            unchecked_item_prefix: console::style(" ".into()),
                            ..Default::default()
                        }),
                        Some("Select items to mark as complete."),
                    )?
                    .into_iter()
                    .for_each(|i| {
                        tasks[i].completed = true;
                    });
                }
            }

            cli::Commands::Remove { task } => {
                if let Some(_task) = task {
                    unimplemented!();
                } else {
                    let selected_indices = io::prompt_user(
                        &tasks_as_strings,
                        // add red x marks on selected items
                        Some(ColorfulTheme {
                            checked_item_prefix: console::style("✗".into()).red(),
                            unchecked_item_prefix: console::style(" ".into()),
                            ..Default::default()
                        }),
                        Some("Select items to remove"),
                    )?;
                    // remove from largest to smallest index so items
                    // don't move around as we delete them
                    selected_indices.iter().rev().for_each(|i| {
                        tasks.remove(*i);
                    });
                }
            }

            cli::Commands::FuckIt => tasks = Vec::new(),
        }
    } else {
        // if no arguments given, print all tasks
        io::print_tasks(&tasks, cols);
    };

    // save tasks back to disc
    let task_yaml = serde_yaml::to_string(&tasks).expect("Unable to serialize todo items.");
    fs::write(todo_file_path, task_yaml).expect("Unable to write to file.");

    Ok(())
}
