pub struct FileType {
    name: String,
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            name: String::from("No filetype"),
        }
    }
}

impl FileType {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn from(file_name: &str) -> Self {
        if file_name.ends_with(".rs") {
            return Self {
                name: String::from("Rust"),
            };
        }

        Self::default()
    }
}
