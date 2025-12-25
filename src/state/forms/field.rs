//! Form field value objects

/// Type-safe field values
#[derive(Debug, Clone)]
pub enum FieldValue {
    Text(String),
    Priority(u32),
}

impl Default for FieldValue {
    fn default() -> Self {
        FieldValue::Text(String::new())
    }
}

/// Represents a single form field with its configuration and value
#[derive(Debug, Clone)]
pub struct FormField {
    pub name: String,
    pub label: String,
    pub value: FieldValue,
    pub is_multiline: bool,
}

impl FormField {
    /// Create a new text field
    pub fn text(name: &str, label: &str, is_multiline: bool) -> Self {
        Self {
            name: name.to_string(),
            label: label.to_string(),
            value: FieldValue::Text(String::new()),
            is_multiline,
        }
    }

    /// Create a new text field with initial value
    pub fn text_with_value(name: &str, label: &str, value: String, is_multiline: bool) -> Self {
        Self {
            name: name.to_string(),
            label: label.to_string(),
            value: FieldValue::Text(value),
            is_multiline,
        }
    }

    /// Create a new priority field
    pub fn priority(name: &str, label: &str) -> Self {
        Self {
            name: name.to_string(),
            label: label.to_string(),
            value: FieldValue::Priority(0),
            is_multiline: false,
        }
    }

    /// Create a new priority field with initial value
    pub fn priority_with_value(name: &str, label: &str, value: u32) -> Self {
        Self {
            name: name.to_string(),
            label: label.to_string(),
            value: FieldValue::Priority(value),
            is_multiline: false,
        }
    }

    /// Get the text value (returns empty string for priority fields)
    pub fn as_text(&self) -> &str {
        match &self.value {
            FieldValue::Text(s) => s,
            FieldValue::Priority(_) => "",
        }
    }

    /// Get the priority value (returns 0 for text fields)
    pub fn as_priority(&self) -> u32 {
        match &self.value {
            FieldValue::Priority(p) => *p,
            FieldValue::Text(_) => 0,
        }
    }

    /// Set the text value
    pub fn set_text(&mut self, value: String) {
        self.value = FieldValue::Text(value);
    }

    /// Set the priority value
    pub fn set_priority(&mut self, value: u32) {
        self.value = FieldValue::Priority(value);
    }

    /// Push a character to the field value
    pub fn push_char(&mut self, c: char) {
        match &mut self.value {
            FieldValue::Text(s) => s.push(c),
            FieldValue::Priority(p) => {
                if let Some(d) = c.to_digit(10) {
                    *p = d;
                }
            }
        }
    }

    /// Remove the last character from the field value
    pub fn pop_char(&mut self) {
        match &mut self.value {
            FieldValue::Text(s) => {
                s.pop();
            }
            FieldValue::Priority(_) => {
                // Priority fields don't support backspace (single digit)
            }
        }
    }

    /// Clear the field value
    pub fn clear(&mut self) {
        match &mut self.value {
            FieldValue::Text(s) => s.clear(),
            FieldValue::Priority(p) => *p = 0,
        }
    }

    /// Get the display value for rendering
    pub fn display_value(&self) -> String {
        match &self.value {
            FieldValue::Text(s) => s.clone(),
            FieldValue::Priority(p) => match *p {
                0 => "Default".to_string(),
                1 => "1 (high)".to_string(),
                2 => "2 (medium)".to_string(),
                3 => "3 (low)".to_string(),
                n => n.to_string(),
            },
        }
    }
}
