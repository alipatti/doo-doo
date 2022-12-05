use crate::structs::Task;
use crate::LINE_CHAR;
use console::measure_text_width;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::{error::Error, fs, vec};

pub(crate) fn prompt_user(
    items: &[impl ToString],
    theme: Option<ColorfulTheme>,
    prompt: Option<&str>,
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let theme = theme.unwrap_or_default();
    let mut selector = MultiSelect::with_theme(&theme);

    if let Some(prompt) = prompt {
        selector.with_prompt(prompt);
    }

    // TODO fix the message printed out after stuff is selected

    Ok(selector.items(items).interact_opt()?.unwrap_or_default())
}

pub(crate) fn read_yaml(filepath: &str) -> Vec<Task> {
    // TODO implement markdown reading/writing
    let todo_file_bytes = match fs::read(filepath) {
        Ok(bytes) => bytes,
        Err(_) => vec::Vec::new(), // file doesn't exist, return empty vec
    };

    serde_yaml::from_slice(&todo_file_bytes).expect("Unable to deserialize todo list.")
}

pub(crate) fn read_markdown(filepath: &str) -> Result<Vec<Task>, Box<dyn Error>> {
    unimplemented!()
}
pub(crate) fn write_markdown(filepath: &str) -> Result<Vec<Task>, Box<dyn Error>> {
    unimplemented!()
}

pub(crate) fn print_tasks(tasks: &Vec<Task>, cols: usize) {
    // print title
    let title = "todo list";
    // FIXME off-by-one padding issue when terminal is an even? width
    let padding = {
        let n_repeats = (cols - measure_text_width(title)) / 2 - 1;
        LINE_CHAR.to_string().repeat(n_repeats)
    };
    println!(
        "{} {} {}",
        padding,
        console::style(title.to_uppercase()).bold().blue(),
        padding
    );

    // print tasks
    for task in tasks {
        // add check mark before item if it's been completed
        let check = if task.completed {
            ColorfulTheme::default().checked_item_prefix
        } else {
            console::style(" ".into())
        };

        println!("{check} {}", task.to_string(cols));
    }

    // print footer
    println!("{}", LINE_CHAR.to_string().repeat(cols as usize));
}

pub(crate) fn terminal_size() -> termsize::Size {
    let default = termsize::Size { rows: 24, cols: 80 };
    termsize::get().unwrap_or(default)
}
