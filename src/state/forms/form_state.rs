//! Form state management and form structs

use super::field::FormField;
use crate::state::{Issue, PullRequest};

/// Trait for common form operations
pub trait Form {
    /// Get the number of fields in this form
    fn field_count(&self) -> usize;

    /// Get the currently active field index
    fn active_field(&self) -> usize;

    /// Set the active field index
    fn set_active_field(&mut self, index: usize);

    /// Move to the next field (wraps around)
    fn next_field(&mut self) {
        let count = self.field_count();
        let current = self.active_field();
        self.set_active_field((current + 1) % count);
    }

    /// Move to the previous field (wraps around)
    fn prev_field(&mut self) {
        let count = self.field_count();
        let current = self.active_field();
        if current == 0 {
            self.set_active_field(count - 1);
        } else {
            self.set_active_field(current - 1);
        }
    }

    /// Get a mutable reference to the active field
    fn get_active_field_mut(&mut self) -> &mut FormField;

    /// Get a reference to a field by index
    fn get_field(&self, index: usize) -> Option<&FormField>;

    /// Clear all form fields
    fn clear(&mut self);
}

/// Enum representing all possible form states
#[derive(Debug, Clone, Default)]
pub enum FormState {
    /// No active form
    #[default]
    None,
    /// Creating a new issue
    IssueCreate(IssueCreateForm),
    /// Editing an existing issue
    IssueEdit(IssueEditForm),
    /// Creating a new pull request
    PrCreate(PrCreateForm),
    /// Editing an existing pull request
    PrEdit(PrEditForm),
    /// Creating a new document
    DocCreate(DocCreateForm),
}

impl FormState {
    /// Check if there is an active form
    pub fn is_active(&self) -> bool {
        !matches!(self, FormState::None)
    }

    /// Get the field count for the current form
    pub fn field_count(&self) -> usize {
        match self {
            FormState::None => 0,
            FormState::IssueCreate(f) => f.field_count(),
            FormState::IssueEdit(f) => f.field_count(),
            FormState::PrCreate(f) => f.field_count(),
            FormState::PrEdit(f) => f.field_count(),
            FormState::DocCreate(f) => f.field_count(),
        }
    }

    /// Move to the next form field
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

    /// Move to the previous form field
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

    /// Get a mutable reference to the active field
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

    /// Get the active field index
    pub fn active_field(&self) -> usize {
        match self {
            FormState::None => 0,
            FormState::IssueCreate(f) => f.active_field(),
            FormState::IssueEdit(f) => f.active_field(),
            FormState::PrCreate(f) => f.active_field(),
            FormState::PrEdit(f) => f.active_field(),
            FormState::DocCreate(f) => f.active_field(),
        }
    }

    /// Set the active field index
    pub fn set_active_field(&mut self, index: usize) {
        match self {
            FormState::None => {}
            FormState::IssueCreate(f) => f.set_active_field(index),
            FormState::IssueEdit(f) => f.set_active_field(index),
            FormState::PrCreate(f) => f.set_active_field(index),
            FormState::PrEdit(f) => f.set_active_field(index),
            FormState::DocCreate(f) => f.set_active_field(index),
        }
    }

    /// Clear the form (reset to None)
    pub fn clear(&mut self) {
        *self = FormState::None;
    }

    /// Check if the active field is multiline
    pub fn is_active_field_multiline(&self) -> bool {
        match self {
            FormState::None => false,
            FormState::IssueCreate(f) => f
                .get_field(f.active_field())
                .map_or(false, |f| f.is_multiline),
            FormState::IssueEdit(f) => f
                .get_field(f.active_field())
                .map_or(false, |f| f.is_multiline),
            FormState::PrCreate(f) => f
                .get_field(f.active_field())
                .map_or(false, |f| f.is_multiline),
            FormState::PrEdit(f) => f
                .get_field(f.active_field())
                .map_or(false, |f| f.is_multiline),
            FormState::DocCreate(f) => f
                .get_field(f.active_field())
                .map_or(false, |f| f.is_multiline),
        }
    }
}

// =============================================================================
// Issue Create Form
// =============================================================================

/// Form for creating a new issue
#[derive(Debug, Clone)]
pub struct IssueCreateForm {
    pub title: FormField,
    pub description: FormField,
    pub priority: FormField,
    pub active_field_index: usize,
}

impl IssueCreateForm {
    /// Create a new empty issue create form
    pub fn new() -> Self {
        Self {
            title: FormField::text("title", "Title", false),
            description: FormField::text("description", "Description", true),
            priority: FormField::priority("priority", "Priority (1-3)"),
            active_field_index: 0,
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
        3
    }

    fn active_field(&self) -> usize {
        self.active_field_index
    }

    fn set_active_field(&mut self, index: usize) {
        self.active_field_index = index.min(self.field_count() - 1);
    }

    fn get_active_field_mut(&mut self) -> &mut FormField {
        match self.active_field_index {
            0 => &mut self.title,
            1 => &mut self.description,
            _ => &mut self.priority,
        }
    }

    fn get_field(&self, index: usize) -> Option<&FormField> {
        match index {
            0 => Some(&self.title),
            1 => Some(&self.description),
            2 => Some(&self.priority),
            _ => None,
        }
    }

    fn clear(&mut self) {
        self.title.clear();
        self.description.clear();
        self.priority.clear();
        self.active_field_index = 0;
    }
}

// =============================================================================
// Issue Edit Form
// =============================================================================

/// Form for editing an existing issue
#[derive(Debug, Clone)]
pub struct IssueEditForm {
    pub title: FormField,
    pub description: FormField,
    pub priority: FormField,
    pub status: FormField,
    pub active_field_index: usize,
    pub issue_id: String,
}

impl IssueEditForm {
    /// Create a new issue edit form from an existing issue
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
            issue_id: issue.id.clone(),
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
        self.active_field_index = index.min(self.field_count() - 1);
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

    fn clear(&mut self) {
        self.title.clear();
        self.description.clear();
        self.priority.clear();
        self.status.clear();
        self.active_field_index = 0;
    }
}

// =============================================================================
// PR Create Form
// =============================================================================

/// Form for creating a new pull request
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
    /// Create a new empty PR create form
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

    /// Get the target branch value, defaulting to "main" if empty
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
        self.active_field_index = index.min(self.field_count() - 1);
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

    fn clear(&mut self) {
        self.title.clear();
        self.description.clear();
        self.source_branch.clear();
        self.target_branch.clear();
        self.priority.clear();
        self.active_field_index = 0;
    }
}

// =============================================================================
// PR Edit Form
// =============================================================================

/// Form for editing an existing pull request
#[derive(Debug, Clone)]
pub struct PrEditForm {
    pub title: FormField,
    pub description: FormField,
    pub source_branch: FormField,
    pub target_branch: FormField,
    pub priority: FormField,
    pub status: FormField,
    pub active_field_index: usize,
    pub pr_id: String,
}

impl PrEditForm {
    /// Create a new PR edit form from an existing pull request
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
            pr_id: pr.id.clone(),
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
        self.active_field_index = index.min(self.field_count() - 1);
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

    fn clear(&mut self) {
        self.title.clear();
        self.description.clear();
        self.source_branch.clear();
        self.target_branch.clear();
        self.priority.clear();
        self.status.clear();
        self.active_field_index = 0;
    }
}

// =============================================================================
// Doc Create Form
// =============================================================================

/// Form for creating a new document
#[derive(Debug, Clone)]
pub struct DocCreateForm {
    pub title: FormField,
    pub content: FormField,
    pub slug: FormField,
    pub active_field_index: usize,
}

impl DocCreateForm {
    /// Create a new empty doc create form
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
        self.active_field_index = index.min(self.field_count() - 1);
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

    fn clear(&mut self) {
        self.title.clear();
        self.content.clear();
        self.slug.clear();
        self.active_field_index = 0;
    }
}
