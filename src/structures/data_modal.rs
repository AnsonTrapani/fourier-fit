use serde_json;

use crate::{DEFAULT_FILENAME, is_file_empty};

pub struct DataModalState {
    pub show_modal: bool,
    pub weight_entry: String,
    pub data: std::collections::HashMap<chrono::DateTime<chrono::Local>, f64>,
    pub selected_datetime: chrono::DateTime<chrono::Local>,
    pub date_status: String,
    pub file: Option<std::path::PathBuf>
}

impl DataModalState {
    pub fn new(f: Option<std::path::PathBuf>) -> Self {
        let dt = chrono::Local::now();
        let mut dt_str = format!("Current date: {}", dt.date_naive());
        let mut map = None;
        let mut ret_f = None;
        if let Some(file) = f {
            let file_str = file.to_str().unwrap_or(DEFAULT_FILENAME);
            let exists_res = std::fs::exists(&file);
            if let Ok(false) = exists_res {
                dt_str = format!("{file_str} does not exist");
            }
            else if exists_res.is_err() {
                dt_str = format!("Unable to verify the existence of {file_str}");
            }
            else {
                if let Ok(ifile) = std::fs::File::open(&file) {
                    if is_file_empty(&file) {
                        map = Some(std::collections::HashMap::new());
                        ret_f = Some(file);
                    } else {
                    match serde_json::from_reader(&ifile) {
                        Ok(m) => {
                            map = Some(m);
                            ret_f = Some(file);
                        }
                        Err(_) => {
                            dt_str = format!("Could not convert {file_str} into HashMap");
                        }
                    }
                }
                }
                else {
                    dt_str = format!("Unable to open file {file_str}");
                }
            }
        }
        Self {
            show_modal: false,
            weight_entry: String::new(),
            data: map.unwrap_or_default(),
            selected_datetime: dt,
            date_status: dt_str,
            file: ret_f
        }
    }

    pub fn log_weight_change(&mut self) -> Result<(), String> {
        let entry = match self.weight_entry.parse::<f64>() {
            Ok(e) => e,
            Err(_) => return Err(format!("{} is not a number.", self.weight_entry))
        };
        self.data.insert(self.selected_datetime, entry);
        Ok(())
    }

    pub fn switch_date_state(&mut self, date_time: chrono::DateTime<chrono::Local>) {
        self.selected_datetime = date_time;
        self.date_status = format!("Current date: {}", self.selected_datetime.date_naive());
        self.weight_entry = match self.data.get(&self.selected_datetime) {
            Some(&v) => v.to_string(),
            None => String::new()
        };
    }
}

impl Default for DataModalState {
    fn default() -> Self {
        DataModalState::new(None)
    }
}

impl Drop for DataModalState {
    fn drop(&mut self) {
        if let Some(ofile) = self.file.as_deref() {
            let file = match std::fs::File::create(ofile) {
                Ok(f) => f,
                Err(_) => {
                    return;
                }
            };
            let _ = serde_json::to_writer(file, &self.data);
        }
    }
}
