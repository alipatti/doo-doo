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
        write!(f, "{}", &self.title)
    }
}

impl Task {
    pub fn to_string(&self, cols: usize) -> String {
        let due_date = self
            .due
            .map_or("no due date".into(), |datetime| datetime.to_string());
        let padding = {
            let used_width =
                console::measure_text_width(&self.title) + console::measure_text_width(&due_date);

            " ".repeat(cols - used_width - 4)
        };
        format!("{self}{padding}({due_date})")
    }
}
