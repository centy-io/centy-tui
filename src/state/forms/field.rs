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

#[cfg(test)]
mod tests {
    use super::*;

    mod field_value {
        use super::*;

        #[test]
        fn test_default_is_empty_text() {
            let value = FieldValue::default();
            if let FieldValue::Text(s) = value {
                assert!(s.is_empty());
            } else {
                panic!("Expected Text variant");
            }
        }

        #[test]
        fn test_text_variant() {
            let value = FieldValue::Text("hello".to_string());
            if let FieldValue::Text(s) = value {
                assert_eq!(s, "hello");
            } else {
                panic!("Expected Text variant");
            }
        }

        #[test]
        fn test_priority_variant() {
            let value = FieldValue::Priority(2);
            if let FieldValue::Priority(p) = value {
                assert_eq!(p, 2);
            } else {
                panic!("Expected Priority variant");
            }
        }
    }

    mod form_field {
        use super::*;

        #[test]
        fn test_text_creates_empty_field() {
            let field = FormField::text("title", "Title", false);
            assert_eq!(field.name, "title");
            assert_eq!(field.label, "Title");
            assert_eq!(field.as_text(), "");
            assert!(!field.is_multiline);
        }

        #[test]
        fn test_text_multiline() {
            let field = FormField::text("desc", "Description", true);
            assert!(field.is_multiline);
        }

        #[test]
        fn test_text_with_value() {
            let field = FormField::text_with_value("title", "Title", "Hello".to_string(), false);
            assert_eq!(field.as_text(), "Hello");
        }

        #[test]
        fn test_priority_creates_zero_value() {
            let field = FormField::priority("priority", "Priority");
            assert_eq!(field.name, "priority");
            assert_eq!(field.label, "Priority");
            assert!(!field.is_multiline);
            if let FieldValue::Priority(p) = &field.value {
                assert_eq!(*p, 0);
            } else {
                panic!("Expected Priority variant");
            }
        }

        #[test]
        fn test_priority_with_value() {
            let field = FormField::priority_with_value("priority", "Priority", 2);
            if let FieldValue::Priority(p) = &field.value {
                assert_eq!(*p, 2);
            } else {
                panic!("Expected Priority variant");
            }
        }

        #[test]
        fn test_as_text_returns_text() {
            let field = FormField::text_with_value("f", "F", "test".to_string(), false);
            assert_eq!(field.as_text(), "test");
        }

        #[test]
        fn test_as_text_returns_empty_for_priority() {
            let field = FormField::priority("p", "P");
            assert_eq!(field.as_text(), "");
        }

        #[test]
        fn test_push_char_to_text() {
            let mut field = FormField::text("f", "F", false);
            field.push_char('a');
            field.push_char('b');
            field.push_char('c');
            assert_eq!(field.as_text(), "abc");
        }

        #[test]
        fn test_push_char_to_priority_digit() {
            let mut field = FormField::priority("p", "P");
            field.push_char('3');
            if let FieldValue::Priority(p) = &field.value {
                assert_eq!(*p, 3);
            }
        }

        #[test]
        fn test_push_char_to_priority_non_digit_ignored() {
            let mut field = FormField::priority("p", "P");
            field.push_char('a');
            if let FieldValue::Priority(p) = &field.value {
                assert_eq!(*p, 0); // Unchanged
            }
        }

        #[test]
        fn test_push_char_to_priority_replaces_value() {
            let mut field = FormField::priority_with_value("p", "P", 1);
            field.push_char('5');
            if let FieldValue::Priority(p) = &field.value {
                assert_eq!(*p, 5); // Replaced, not appended
            }
        }

        #[test]
        fn test_pop_char_from_text() {
            let mut field = FormField::text_with_value("f", "F", "abc".to_string(), false);
            field.pop_char();
            assert_eq!(field.as_text(), "ab");
        }

        #[test]
        fn test_pop_char_from_empty_text() {
            let mut field = FormField::text("f", "F", false);
            field.pop_char(); // Should not panic
            assert_eq!(field.as_text(), "");
        }

        #[test]
        fn test_pop_char_from_priority_no_op() {
            let mut field = FormField::priority_with_value("p", "P", 3);
            field.pop_char(); // Should do nothing
            if let FieldValue::Priority(p) = &field.value {
                assert_eq!(*p, 3); // Unchanged
            }
        }

        #[test]
        fn test_display_value_text() {
            let field = FormField::text_with_value("f", "F", "hello world".to_string(), false);
            assert_eq!(field.display_value(), "hello world");
        }

        #[test]
        fn test_display_value_priority_0() {
            let field = FormField::priority("p", "P");
            assert_eq!(field.display_value(), "Default");
        }

        #[test]
        fn test_display_value_priority_1() {
            let field = FormField::priority_with_value("p", "P", 1);
            assert_eq!(field.display_value(), "1 (high)");
        }

        #[test]
        fn test_display_value_priority_2() {
            let field = FormField::priority_with_value("p", "P", 2);
            assert_eq!(field.display_value(), "2 (medium)");
        }

        #[test]
        fn test_display_value_priority_3() {
            let field = FormField::priority_with_value("p", "P", 3);
            assert_eq!(field.display_value(), "3 (low)");
        }

        #[test]
        fn test_display_value_priority_other() {
            let field = FormField::priority_with_value("p", "P", 7);
            assert_eq!(field.display_value(), "7");
        }
    }
}
