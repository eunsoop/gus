
pub(crate) struct ConfigFile {
    lines: Vec<String>,
}

impl ConfigFile {
    pub(crate) fn new(path: &str) -> Self {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let lines = content.lines().map(|line| line.to_string()).collect();
        Self { lines }
    }

    pub(crate) fn find_line(&self, section: &str, key: &str) -> Option<usize> {
        self.lines.iter().position(|line| line.replace(" ", "").contains(format!("[{}]", section).as_str()))
            .and_then(|start| {
                self.lines[start+1..]
                    .iter()
                    .position(|line| line.trim().starts_with(format!("{}", key).as_str()) || line.starts_with('['))
                    .filter(|line| !self.lines[start + 1 + line].starts_with('['))
                    .map(|pos| start + 1 + pos)
            })
    }

    pub(crate) fn find_section(&self, section: &str) -> Option<usize> {
        self.lines.iter().position(|line| line.replace(" ", "").contains(format!("[{}]", section).as_str()))
    }

    pub(crate) fn append_line(&mut self, section: &str, key: &str, value: &str) {
        if let Some(section_line) = self.lines.iter().position(|line| line.trim().contains(format!("[{}]", section).as_str())) {
            let insert_pos = self.lines[section_line..]
                .iter()
                .position(|line| line.starts_with('['))
                .map_or(self.lines.len(), |pos| section_line + pos);
            self.lines.insert(insert_pos+1, format!("\t{} = {}", key, value));
        } else {
            self.lines.push(format!("[{}]", section));
            self.lines.push(format!("\t{} = {}", key, value));
        }
    }

    pub(crate) fn update_line(&mut self, pos: usize, key: &str, new_value: &str) {
        self.lines[pos] = format!("\t{} = {}", key, new_value);
    }

    pub (crate) fn remove_line(&mut self, line_num: usize) {
        if line_num < self.lines.len() {
            self.lines.remove(line_num);
        }
    }

    pub(crate) fn save(&self, path: &str) -> std::io::Result<()> {
        std::fs::write(path, self.lines.join("\n").as_str()).expect("Failed to write to file");
        Ok(())
    }
}