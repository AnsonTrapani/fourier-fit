pub struct DataModalState {
    pub show_modal: bool,
    pub weight_entry: String,
    pub data: std::collections::HashMap<chrono::DateTime<chrono::Local>, f64>,
    pub selected_datetime: chrono::DateTime<chrono::Local>
}

impl DataModalState {
    pub fn new() -> Self {
        Self {
            show_modal: false,
            weight_entry: String::new(),
            data: std::collections::HashMap::new(),
            selected_datetime: chrono::Local::now()
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

    pub fn load_file(file: std::path::PathBuf) -> Result<std::collections::HashMap<chrono::DateTime<chrono::Local>, f64>, String> {
        let file_str = file.to_str().unwrap_or("file");
        let exists_res = std::fs::exists(&file);
        if let Ok(false) = exists_res {
            return Err(format!("{file_str} does not exist"));
        }
        if exists_res.is_err() {
            return Err(format!("Unabele to verify the existence of {file_str}"));
        }
        Ok(std::collections::HashMap::new())
    }

    pub fn switch_date_display(&mut self) {
        self.weight_entry = match self.data.get(&self.selected_datetime) {
            Some(&v) => v.to_string(),
            None => String::new()
        };
    }
}

impl Default for DataModalState {
    fn default() -> Self {
        DataModalState::new()
    }
}
