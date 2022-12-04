#![warn(clippy::pedantic)]

use clap::Parser as _;
use dialoguer::theme::ColorfulTheme;
use std::{error::Error, fs, process::Command};

// horizontal bar, not a dash (-)
const LINE_CHAR: char = '─';

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();

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
    let mut tasks = io::read_task_file(&todo_file_path);

    if let Some(command) = args.command {
        match command {
            // print all tasks
            cli::Commands::List => io::print_tasks(&tasks),

            cli::Commands::Open => {
                Command::new("open")
                    .arg(&todo_file_path)
                    .spawn()
                    .expect("Failed to open file.");
            }

            // add task
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

            // mark task(s) as done
            cli::Commands::Done { item } => {
                if let Some(_item) = item {
                    unimplemented!();
                    // filter tasks based on item
                    // if there's one thing left, remove it.
                    // if there are multiple matches, prompt user to select from them
                } else {
                    for i in io::user_select_indices(
                        &tasks,
                        // remove gray check mark for unselected items
                        Some(ColorfulTheme {
                            unchecked_item_prefix: console::style(" ".into()),
                            ..Default::default()
                        }),
                        Some("Select items to mark as complete."),
                    )? {
                        tasks[i].completed = true;
                    }
                }
            }

            // remove task(s)
            cli::Commands::Remove { task: item } => {
                if let Some(_item) = item {
                    unimplemented!();
                } else {
                    let selected_indices = io::user_select_indices(
                        &tasks,
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
                    for i in selected_indices.iter().rev() {
                        tasks.remove(*i);
                    }
                }
            }

            // remove all tasks
            cli::Commands::FuckIt => tasks = Vec::new(),
        }
    } else {
        // if no arguments given, print all tasks
        io::print_tasks(&tasks);
    };

    // save tasks back to disc
    // this is a self-proclaimed shitty app, so we're allowed to use .expect()
    let task_yaml = serde_yaml::to_string(&tasks).expect("Unable to serialize todo items.");
    fs::write(todo_file_path, task_yaml).expect("Unable to write to file.");

    Ok(())
}

mod structs {
    use chrono::{DateTime, Local};
    use serde::{Deserialize, Serialize};
    use std::fmt;

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct Task {
        pub(crate) title: String,

        pub(crate) details: Option<Vec<String>>,

        pub(crate) due: Option<DateTime<Local>>,

        #[serde(default)]
        pub(crate) completed: bool,
    }

    impl fmt::Display for Task {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            // TODO implement better display
            // let size = termsize::get().unwrap_or(Size { rows: 24, cols: 80 });

            write!(f, "{}", self.title)
        }
    }
}

mod cli {
    use clap::{Parser, Subcommand, ValueEnum};
    use humantime::Duration;

    /// 💩 the shitty todo app
    #[derive(Parser, Debug)]
    pub(crate) struct Args {
        #[command(subcommand)]
        pub(crate) command: Option<Commands>,

        /// Specify a todo file (default: ./todo.yaml)
        #[arg(short, long)]
        pub(crate) file: Option<String>,

        /// Use the global todo file (~/todo.yaml)
        #[arg(short, long)]
        pub(crate) global: bool,

        /// Show completed items
        #[arg(short, long)]
        pub(crate) completed: bool,

        /// How to sort the todo items
        #[arg(value_enum, short, long)]
        pub(crate) sort: Option<SortOptions>,

        /// Include todo items parsed from the workspace rooted in the current directory.
        ///
        /// By default, this command searches for all lines containing the string TODO.
        #[arg(short, long)]
        pub(crate) include_code: bool,
    }

    #[derive(Subcommand, Debug)]
    pub(crate) enum Commands {
        /// Print all tasks. Equivalent to passing no command.
        List,

        /// Open the markdown file containing your todo list.
        Open,

        /// Add a task
        Add {
            /// The title of the task.
            title: String,

            #[arg(short, long)]
            /// An optional list of
            details: Option<Vec<String>>,

            #[arg(short = 't', long)]
            /// When the task is due.
            ///
            /// Formatted as a duration representing how much time you have to complete the task.
            ///
            /// e.g. '1 week', '3hours 42minutes', '8 months',
            due: Option<Duration>,
        },

        /// Mark a task as completed.
        ///
        /// Pass the name of a task to mark it as complete.
        /// If there are multiple matching tasks, you will be asked to choose between them.
        /// If no argument is passed, you will be presented with a menu to select the tasks to mark as complete.
        Done { item: Option<String> },

        /// Delete a task.
        ///
        /// Pass the name of a task to remove it.
        /// If there are multiple matching tasks, you will be asked to choose between them.
        /// If no argument is passed, you will be presented with a menu to select the tasks to delete.
        Remove { task: Option<String> },

        /// Delete all tasks.
        FuckIt,
    }

    #[derive(Debug, Clone, ValueEnum, Default)]
    pub(crate) enum SortOptions {
        #[default]
        Date,
        Title,
    }
}

mod io {
    use crate::LINE_CHAR;

    use super::structs::Task;
    use console::measure_text_width;
    use dialoguer::{theme::ColorfulTheme, MultiSelect};
    use std::{fs, io, vec};

    pub(crate) fn user_select_indices(
        items: &[impl ToString],
        theme: Option<ColorfulTheme>,
        prompt: Option<&str>,
    ) -> io::Result<Vec<usize>> {
        let theme = theme.unwrap_or_default();
        let mut selector = MultiSelect::with_theme(&theme);

        if let Some(prompt) = prompt {
            selector.with_prompt(prompt);
        }

        // TODO let user use the esc key to get out
        selector.items(items).interact()
    }

    pub(crate) fn read_task_file(filepath: &str) -> Vec<Task> {
        let todo_file_bytes = match fs::read(filepath) {
            Ok(bytes) => bytes,
            Err(_) => vec::Vec::new(), // file doesn't exist, return empty vec
        };

        serde_yaml::from_slice(&todo_file_bytes).expect("Unable to deserialize todo list.")
    }

    pub(crate) fn print_tasks(tasks: &Vec<Task>) {
        let terminal_size = termsize::get().unwrap_or(termsize::Size { rows: 24, cols: 80 });

        // print title
        let title = "todo list";
        let padding = {
            let n_repeats = (terminal_size.cols as usize - measure_text_width(title)) / 2 - 1;
            LINE_CHAR.to_string().repeat(n_repeats)
        };
        println!(
            "{} {} {}",
            padding,
            console::style(title.to_uppercase()).bold().blue(),
            padding
        );

        // print tasks
        for item in tasks {
            // add check mark before item if it's been completed
            let check = if item.completed {
                ColorfulTheme::default().checked_item_prefix
            } else {
                console::style(" ".into())
            };
            println!("{} {item}", check);
        }

        // print footer
        println!(
            "{}",
            LINE_CHAR.to_string().repeat(terminal_size.cols as usize)
        );
    }
}
