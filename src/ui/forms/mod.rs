//! Form components for creating and editing issues, PRs, and docs
//!
//! This module provides a modular DDD-based structure for form rendering.

mod doc_form;
mod field_renderer;
mod issue_form;
mod pr_form;

// Re-export with original function names for API compatibility
pub use doc_form::draw_create as draw_doc_create;
pub use issue_form::draw_create as draw_issue_create;
pub use issue_form::draw_edit as draw_issue_edit;
pub use pr_form::draw_create as draw_pr_create;
pub use pr_form::draw_edit as draw_pr_edit;
