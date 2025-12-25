//! Form state management and form structs

use super::field::FormField;
use crate::state::{Issue, PullRequest};

/// Trait for common form operations
pub trait Form {
    fn field_count(&self) -> usize;
    fn active_field(&self) -> usize;
    fn set_active_field(&mut self, index: usize);
    fn next_field(&mut self) {
        let count = self.field_count();
        let current = self.active_field();
        self.set_active_field((current + 1) % count);
    }
    fn prev_field(&mut self) {
        let count = self.field_count();
        let current = self.active_field();
        if current == 0 {
            self.set_active_field(count - 1);
        } else {
            self.set_active_field(current - 1);
        }
    }
    fn get_active_field_mut(&mut self) -> &mut FormField;
    fn get_field(&self, index: usize) -> Option<&FormField>;
}

/// Enum representing all possible form states
#[derive(Debug, Clone, Default)]
pub enum FormState {
    #[default]
    None,
    IssueCreate(IssueCreateForm),
    IssueEdit(IssueEditForm),
    PrCreate(PrCreateForm),
    PrEdit(PrEditForm),
    DocCreate(DocCreateForm),
}

impl FormState {
    pub fn next_field(&mut self) {
        match self {
            FormState::None => {}
            FormState::IssueCreate(f) => f.next_field(),
            FormState::IssueEdit(f) => f.next_field(),
            FormState::PrCreate(f) => f.next_field(),
            FormState::PrEdit(f) => f.next_field(),
            FormState::DocCreate(f) => f.next_field(),
        }
    }

    pub fn prev_field(&mut self) {
        match self {
            FormState::None => {}
            FormState::IssueCreate(f) => f.prev_field(),
            FormState::IssueEdit(f) => f.prev_field(),
            FormState::PrCreate(f) => f.prev_field(),
            FormState::PrEdit(f) => f.prev_field(),
            FormState::DocCreate(f) => f.prev_field(),
        }
    }

    pub fn get_active_field_mut(&mut self) -> Option<&mut FormField> {
        match self {
            FormState::None => None,
            FormState::IssueCreate(f) => Some(f.get_active_field_mut()),
            FormState::IssueEdit(f) => Some(f.get_active_field_mut()),
            FormState::PrCreate(f) => Some(f.get_active_field_mut()),
            FormState::PrEdit(f) => Some(f.get_active_field_mut()),
            FormState::DocCreate(f) => Some(f.get_active_field_mut()),
        }
    }

    pub fn is_active_field_multiline(&self) -> bool {
        match self {
            FormState::None => false,
            FormState::IssueCreate(f) => f
                .get_field(f.active_field())
                .is_some_and(|f| f.is_multiline),
            FormState::IssueEdit(f) => f
                .get_field(f.active_field())
                .is_some_and(|f| f.is_multiline),
            FormState::PrCreate(f) => f
                .get_field(f.active_field())
                .is_some_and(|f| f.is_multiline),
            FormState::PrEdit(f) => f
                .get_field(f.active_field())
                .is_some_and(|f| f.is_multiline),
            FormState::DocCreate(f) => f
                .get_field(f.active_field())
                .is_some_and(|f| f.is_multiline),
        }
    }
}

// Issue Create Form
#[derive(Debug, Clone)]
pub struct IssueCreateForm {
    pub title: FormField,
    pub description: FormField,
    pub priority: FormField,
    pub active_field_index: usize,
    /// Which button is selected when on the buttons row (0=Cancel, 1=Draft, 2=Create&New, 3=Create)
    pub selected_button: usize,
}

impl IssueCreateForm {
    pub fn new() -> Self {
        Self {
            title: FormField::text("title", "Title", false),
            description: FormField::text("description", "Description", true),
            priority: FormField::priority("priority", "Priority (1-3)"),
            active_field_index: 0,
            selected_button: 3, // Default to "Create" button
        }
    }

    /// Returns true if the buttons row is currently active
    pub fn is_buttons_row_active(&self) -> bool {
        self.active_field_index == 3
    }

    /// Move to the next button (wraps around)
    pub fn next_button(&mut self) {
        self.selected_button = (self.selected_button + 1) % 4;
    }

    /// Move to the previous button (wraps around)
    pub fn prev_button(&mut self) {
        if self.selected_button == 0 {
            self.selected_button = 3;
        } else {
            self.selected_button -= 1;
        }
    }
}

impl Default for IssueCreateForm {
    fn default() -> Self {
        Self::new()
    }
}

impl Form for IssueCreateForm {
    fn field_count(&self) -> usize {
        4 // title, description, priority, buttons
    }
    fn active_field(&self) -> usize {
        self.active_field_index
    }
    fn set_active_field(&mut self, index: usize) {
        self.active_field_index = index.min(3);
    }
    fn get_active_field_mut(&mut self) -> &mut FormField {
        match self.active_field_index {
            0 => &mut self.title,
            1 => &mut self.description,
            // For buttons row (index 3), return priority as dummy (won't be used for text input)
            _ => &mut self.priority,
        }
    }
    fn get_field(&self, index: usize) -> Option<&FormField> {
        match index {
            0 => Some(&self.title),
            1 => Some(&self.description),
            2 => Some(&self.priority),
            // Index 3 is buttons row, no FormField for it
            _ => None,
        }
    }
}

// Issue Edit Form
#[derive(Debug, Clone)]
pub struct IssueEditForm {
    pub title: FormField,
    pub description: FormField,
    pub priority: FormField,
    pub status: FormField,
    pub active_field_index: usize,
}

impl IssueEditForm {
    pub fn from_issue(issue: &Issue) -> Self {
        Self {
            title: FormField::text_with_value("title", "Title", issue.title.clone(), false),
            description: FormField::text_with_value(
                "description",
                "Description",
                issue.description.clone(),
                true,
            ),
            priority: FormField::priority_with_value(
                "priority",
                "Priority (1-3)",
                issue.metadata.priority,
            ),
            status: FormField::text_with_value(
                "status",
                "Status",
                issue.metadata.status.clone(),
                false,
            ),
            active_field_index: 0,
        }
    }
}

impl Form for IssueEditForm {
    fn field_count(&self) -> usize {
        4
    }
    fn active_field(&self) -> usize {
        self.active_field_index
    }
    fn set_active_field(&mut self, index: usize) {
        self.active_field_index = index.min(3);
    }
    fn get_active_field_mut(&mut self) -> &mut FormField {
        match self.active_field_index {
            0 => &mut self.title,
            1 => &mut self.description,
            2 => &mut self.priority,
            _ => &mut self.status,
        }
    }
    fn get_field(&self, index: usize) -> Option<&FormField> {
        match index {
            0 => Some(&self.title),
            1 => Some(&self.description),
            2 => Some(&self.priority),
            3 => Some(&self.status),
            _ => None,
        }
    }
}

// PR Create Form
#[derive(Debug, Clone)]
pub struct PrCreateForm {
    pub title: FormField,
    pub description: FormField,
    pub source_branch: FormField,
    pub target_branch: FormField,
    pub priority: FormField,
    pub active_field_index: usize,
}

impl PrCreateForm {
    pub fn new() -> Self {
        Self {
            title: FormField::text("title", "Title", false),
            description: FormField::text("description", "Description", true),
            source_branch: FormField::text("source_branch", "Source Branch", false),
            target_branch: FormField::text("target_branch", "Target Branch", false),
            priority: FormField::priority("priority", "Priority (1-3)"),
            active_field_index: 0,
        }
    }

    pub fn target_branch_or_default(&self) -> &str {
        let value = self.target_branch.as_text();
        if value.is_empty() {
            "main"
        } else {
            value
        }
    }
}

impl Default for PrCreateForm {
    fn default() -> Self {
        Self::new()
    }
}

impl Form for PrCreateForm {
    fn field_count(&self) -> usize {
        5
    }
    fn active_field(&self) -> usize {
        self.active_field_index
    }
    fn set_active_field(&mut self, index: usize) {
        self.active_field_index = index.min(4);
    }
    fn get_active_field_mut(&mut self) -> &mut FormField {
        match self.active_field_index {
            0 => &mut self.title,
            1 => &mut self.description,
            2 => &mut self.source_branch,
            3 => &mut self.target_branch,
            _ => &mut self.priority,
        }
    }
    fn get_field(&self, index: usize) -> Option<&FormField> {
        match index {
            0 => Some(&self.title),
            1 => Some(&self.description),
            2 => Some(&self.source_branch),
            3 => Some(&self.target_branch),
            4 => Some(&self.priority),
            _ => None,
        }
    }
}

// PR Edit Form
#[derive(Debug, Clone)]
pub struct PrEditForm {
    pub title: FormField,
    pub description: FormField,
    pub source_branch: FormField,
    pub target_branch: FormField,
    pub priority: FormField,
    pub status: FormField,
    pub active_field_index: usize,
}

impl PrEditForm {
    pub fn from_pr(pr: &PullRequest) -> Self {
        Self {
            title: FormField::text_with_value("title", "Title", pr.title.clone(), false),
            description: FormField::text_with_value(
                "description",
                "Description",
                pr.description.clone(),
                true,
            ),
            source_branch: FormField::text_with_value(
                "source_branch",
                "Source Branch",
                pr.metadata.source_branch.clone(),
                false,
            ),
            target_branch: FormField::text_with_value(
                "target_branch",
                "Target Branch",
                pr.metadata.target_branch.clone(),
                false,
            ),
            priority: FormField::priority_with_value("priority", "Priority", pr.metadata.priority),
            status: FormField::text_with_value(
                "status",
                "Status",
                pr.metadata.status.clone(),
                false,
            ),
            active_field_index: 0,
        }
    }
}

impl Form for PrEditForm {
    fn field_count(&self) -> usize {
        6
    }
    fn active_field(&self) -> usize {
        self.active_field_index
    }
    fn set_active_field(&mut self, index: usize) {
        self.active_field_index = index.min(5);
    }
    fn get_active_field_mut(&mut self) -> &mut FormField {
        match self.active_field_index {
            0 => &mut self.title,
            1 => &mut self.description,
            2 => &mut self.source_branch,
            3 => &mut self.target_branch,
            4 => &mut self.priority,
            _ => &mut self.status,
        }
    }
    fn get_field(&self, index: usize) -> Option<&FormField> {
        match index {
            0 => Some(&self.title),
            1 => Some(&self.description),
            2 => Some(&self.source_branch),
            3 => Some(&self.target_branch),
            4 => Some(&self.priority),
            5 => Some(&self.status),
            _ => None,
        }
    }
}

// Doc Create Form
#[derive(Debug, Clone)]
pub struct DocCreateForm {
    pub title: FormField,
    pub content: FormField,
    pub slug: FormField,
    pub active_field_index: usize,
}

impl DocCreateForm {
    pub fn new() -> Self {
        Self {
            title: FormField::text("title", "Title", false),
            content: FormField::text("content", "Content (Markdown)", true),
            slug: FormField::text("slug", "Slug (optional)", false),
            active_field_index: 0,
        }
    }
}

impl Default for DocCreateForm {
    fn default() -> Self {
        Self::new()
    }
}

impl Form for DocCreateForm {
    fn field_count(&self) -> usize {
        3
    }
    fn active_field(&self) -> usize {
        self.active_field_index
    }
    fn set_active_field(&mut self, index: usize) {
        self.active_field_index = index.min(2);
    }
    fn get_active_field_mut(&mut self) -> &mut FormField {
        match self.active_field_index {
            0 => &mut self.title,
            1 => &mut self.content,
            _ => &mut self.slug,
        }
    }
    fn get_field(&self, index: usize) -> Option<&FormField> {
        match index {
            0 => Some(&self.title),
            1 => Some(&self.content),
            2 => Some(&self.slug),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{IssueMetadata, PrMetadata};
    use chrono::Utc;
    use std::collections::HashMap;

    // Helper function to create test Issue
    fn create_test_issue() -> Issue {
        Issue {
            id: "test-id".to_string(),
            display_number: 1,
            title: "Test Issue".to_string(),
            description: "Test Description".to_string(),
            metadata: IssueMetadata {
                status: "open".to_string(),
                priority: 2,
                priority_label: Some("medium".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                custom_fields: HashMap::new(),
            },
        }
    }

    // Helper function to create test PR
    fn create_test_pr() -> PullRequest {
        PullRequest {
            id: "test-pr".to_string(),
            display_number: 1,
            title: "Test PR".to_string(),
            description: "Test PR Description".to_string(),
            metadata: PrMetadata {
                status: "open".to_string(),
                priority: 1,
                priority_label: Some("high".to_string()),
                source_branch: "feature".to_string(),
                target_branch: "main".to_string(),
                linked_issues: vec![],
                reviewers: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                merged_at: None,
                closed_at: None,
                custom_fields: HashMap::new(),
            },
        }
    }

    mod form_state_enum {
        use super::*;

        #[test]
        fn test_default_is_none() {
            let state = FormState::default();
            assert!(matches!(state, FormState::None));
        }

        #[test]
        fn test_next_field_on_none_is_noop() {
            let mut state = FormState::None;
            state.next_field(); // Should not panic
        }

        #[test]
        fn test_prev_field_on_none_is_noop() {
            let mut state = FormState::None;
            state.prev_field(); // Should not panic
        }

        #[test]
        fn test_get_active_field_mut_none_returns_none() {
            let mut state = FormState::None;
            assert!(state.get_active_field_mut().is_none());
        }

        #[test]
        fn test_is_active_field_multiline_none_is_false() {
            let state = FormState::None;
            assert!(!state.is_active_field_multiline());
        }

        #[test]
        fn test_issue_create_variant() {
            let form = IssueCreateForm::new();
            let state = FormState::IssueCreate(form);
            assert!(matches!(state, FormState::IssueCreate(_)));
        }

        #[test]
        fn test_next_field_cycles_through_form() {
            let mut state = FormState::IssueCreate(IssueCreateForm::new());
            if let FormState::IssueCreate(ref f) = state {
                assert_eq!(f.active_field_index, 0);
            }
            state.next_field();
            if let FormState::IssueCreate(ref f) = state {
                assert_eq!(f.active_field_index, 1);
            }
        }

        #[test]
        fn test_get_active_field_mut_returns_field() {
            let mut state = FormState::IssueCreate(IssueCreateForm::new());
            let field = state.get_active_field_mut();
            assert!(field.is_some());
            assert_eq!(field.unwrap().name, "title");
        }

        #[test]
        fn test_is_active_field_multiline_title_is_false() {
            let state = FormState::IssueCreate(IssueCreateForm::new());
            assert!(!state.is_active_field_multiline());
        }

        #[test]
        fn test_is_active_field_multiline_description_is_true() {
            let mut form = IssueCreateForm::new();
            form.active_field_index = 1; // description
            let state = FormState::IssueCreate(form);
            assert!(state.is_active_field_multiline());
        }
    }

    mod issue_create_form {
        use super::*;

        #[test]
        fn test_new_has_correct_defaults() {
            let form = IssueCreateForm::new();
            assert_eq!(form.active_field_index, 0);
            assert_eq!(form.selected_button, 3); // Create button
            assert_eq!(form.title.name, "title");
            assert_eq!(form.description.name, "description");
            assert_eq!(form.priority.name, "priority");
        }

        #[test]
        fn test_default_equals_new() {
            let new = IssueCreateForm::new();
            let default = IssueCreateForm::default();
            assert_eq!(new.active_field_index, default.active_field_index);
            assert_eq!(new.selected_button, default.selected_button);
        }

        #[test]
        fn test_field_count() {
            let form = IssueCreateForm::new();
            assert_eq!(form.field_count(), 4);
        }

        #[test]
        fn test_is_buttons_row_active() {
            let mut form = IssueCreateForm::new();
            assert!(!form.is_buttons_row_active());
            form.active_field_index = 3;
            assert!(form.is_buttons_row_active());
        }

        #[test]
        fn test_next_button_wraps() {
            let mut form = IssueCreateForm::new();
            form.selected_button = 3;
            form.next_button();
            assert_eq!(form.selected_button, 0);
        }

        #[test]
        fn test_prev_button_wraps() {
            let mut form = IssueCreateForm::new();
            form.selected_button = 0;
            form.prev_button();
            assert_eq!(form.selected_button, 3);
        }

        #[test]
        fn test_next_field_cycles() {
            let mut form = IssueCreateForm::new();
            for _ in 0..4 {
                form.next_field();
            }
            assert_eq!(form.active_field_index, 0); // Wrapped back
        }

        #[test]
        fn test_prev_field_cycles() {
            let mut form = IssueCreateForm::new();
            form.prev_field();
            assert_eq!(form.active_field_index, 3); // Wrapped to last
        }

        #[test]
        fn test_get_field_returns_correct_fields() {
            let form = IssueCreateForm::new();
            assert_eq!(form.get_field(0).unwrap().name, "title");
            assert_eq!(form.get_field(1).unwrap().name, "description");
            assert_eq!(form.get_field(2).unwrap().name, "priority");
            assert!(form.get_field(3).is_none()); // buttons row
            assert!(form.get_field(4).is_none());
        }

        #[test]
        fn test_set_active_field_clamps() {
            let mut form = IssueCreateForm::new();
            form.set_active_field(100);
            assert_eq!(form.active_field_index, 3);
        }
    }

    mod issue_edit_form {
        use super::*;

        #[test]
        fn test_from_issue_loads_values() {
            let issue = create_test_issue();
            let form = IssueEditForm::from_issue(&issue);

            assert_eq!(form.title.as_text(), "Test Issue");
            assert_eq!(form.description.as_text(), "Test Description");
            assert_eq!(form.status.as_text(), "open");
            assert_eq!(form.active_field_index, 0);
        }

        #[test]
        fn test_field_count() {
            let issue = create_test_issue();
            let form = IssueEditForm::from_issue(&issue);
            assert_eq!(form.field_count(), 4);
        }

        #[test]
        fn test_get_field_returns_correct_fields() {
            let issue = create_test_issue();
            let form = IssueEditForm::from_issue(&issue);
            assert_eq!(form.get_field(0).unwrap().name, "title");
            assert_eq!(form.get_field(1).unwrap().name, "description");
            assert_eq!(form.get_field(2).unwrap().name, "priority");
            assert_eq!(form.get_field(3).unwrap().name, "status");
            assert!(form.get_field(4).is_none());
        }
    }

    mod pr_create_form {
        use super::*;

        #[test]
        fn test_new_has_correct_defaults() {
            let form = PrCreateForm::new();
            assert_eq!(form.active_field_index, 0);
            assert_eq!(form.title.as_text(), "");
            assert_eq!(form.source_branch.as_text(), "");
            assert_eq!(form.target_branch.as_text(), "");
        }

        #[test]
        fn test_field_count() {
            let form = PrCreateForm::new();
            assert_eq!(form.field_count(), 5);
        }

        #[test]
        fn test_target_branch_or_default_empty() {
            let form = PrCreateForm::new();
            assert_eq!(form.target_branch_or_default(), "main");
        }

        #[test]
        fn test_target_branch_or_default_with_value() {
            let mut form = PrCreateForm::new();
            form.target_branch.push_char('d');
            form.target_branch.push_char('e');
            form.target_branch.push_char('v');
            assert_eq!(form.target_branch_or_default(), "dev");
        }

        #[test]
        fn test_get_field_returns_correct_fields() {
            let form = PrCreateForm::new();
            assert_eq!(form.get_field(0).unwrap().name, "title");
            assert_eq!(form.get_field(1).unwrap().name, "description");
            assert_eq!(form.get_field(2).unwrap().name, "source_branch");
            assert_eq!(form.get_field(3).unwrap().name, "target_branch");
            assert_eq!(form.get_field(4).unwrap().name, "priority");
            assert!(form.get_field(5).is_none());
        }
    }

    mod pr_edit_form {
        use super::*;

        #[test]
        fn test_from_pr_loads_values() {
            let pr = create_test_pr();
            let form = PrEditForm::from_pr(&pr);

            assert_eq!(form.title.as_text(), "Test PR");
            assert_eq!(form.description.as_text(), "Test PR Description");
            assert_eq!(form.source_branch.as_text(), "feature");
            assert_eq!(form.target_branch.as_text(), "main");
            assert_eq!(form.status.as_text(), "open");
        }

        #[test]
        fn test_field_count() {
            let pr = create_test_pr();
            let form = PrEditForm::from_pr(&pr);
            assert_eq!(form.field_count(), 6);
        }

        #[test]
        fn test_get_field_returns_correct_fields() {
            let pr = create_test_pr();
            let form = PrEditForm::from_pr(&pr);
            assert_eq!(form.get_field(0).unwrap().name, "title");
            assert_eq!(form.get_field(1).unwrap().name, "description");
            assert_eq!(form.get_field(2).unwrap().name, "source_branch");
            assert_eq!(form.get_field(3).unwrap().name, "target_branch");
            assert_eq!(form.get_field(4).unwrap().name, "priority");
            assert_eq!(form.get_field(5).unwrap().name, "status");
            assert!(form.get_field(6).is_none());
        }
    }

    mod doc_create_form {
        use super::*;

        #[test]
        fn test_new_has_correct_defaults() {
            let form = DocCreateForm::new();
            assert_eq!(form.active_field_index, 0);
            assert_eq!(form.title.as_text(), "");
            assert_eq!(form.content.as_text(), "");
            assert_eq!(form.slug.as_text(), "");
        }

        #[test]
        fn test_field_count() {
            let form = DocCreateForm::new();
            assert_eq!(form.field_count(), 3);
        }

        #[test]
        fn test_content_is_multiline() {
            let form = DocCreateForm::new();
            assert!(form.get_field(1).unwrap().is_multiline);
        }

        #[test]
        fn test_get_field_returns_correct_fields() {
            let form = DocCreateForm::new();
            assert_eq!(form.get_field(0).unwrap().name, "title");
            assert_eq!(form.get_field(1).unwrap().name, "content");
            assert_eq!(form.get_field(2).unwrap().name, "slug");
            assert!(form.get_field(3).is_none());
        }

        #[test]
        fn test_set_active_field_clamps() {
            let mut form = DocCreateForm::new();
            form.set_active_field(100);
            assert_eq!(form.active_field_index, 2);
        }
    }
}
