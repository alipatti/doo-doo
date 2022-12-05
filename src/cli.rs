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
        /// The nitty gritty.
        ///
        /// Multiple details can be passed by using the -d flag more than once.
        /// e.g.
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
    Done { task: Option<String> },

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
