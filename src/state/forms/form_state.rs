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

// Issue Create Form
#[derive(Debug, Clone)]
pub struct IssueCreateForm {
    pub title: FormField,
    pub description: FormField,
    pub priority: FormField,
    pub active_field_index: usize,
}

impl IssueCreateForm {
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
        self.active_field_index = index.min(2);
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
