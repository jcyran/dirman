pub struct UserInput {
    input_value: String,
    input_index: usize,
}

impl Default for UserInput {
    fn default() -> Self {
        Self {
            input_value: String::default(),
            input_index: 0,
        }
    }
}

impl UserInput {
    pub fn get_input_value(&self) -> String {
        self.input_value.clone()
    }

    pub fn new(start_string: String) -> Self {
        let new_value = start_string.clone();
        let new_index = new_value.len();

        Self {
            input_value: new_value,
            input_index: new_index,
        }
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input_value.insert(index, new_char);
    }

    pub fn byte_index(&self) -> usize {
        self.input_value
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.input_index)
            .unwrap_or(self.input_value.len())
    }

    pub fn delete_char(&mut self) {
        if self.input_index != 0 {
            self.input_value.pop();
        }
    }
}

