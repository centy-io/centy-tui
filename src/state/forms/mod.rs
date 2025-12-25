//! Form domain layer
//!
//! This module provides type-safe form handling for create/edit views.
//! Currently scaffolding for future integration.

#![allow(dead_code)]

mod field;
mod form_state;

pub use field::FormField;
pub use form_state::{
    DocCreateForm, FormState, IssueCreateForm, IssueEditForm, PrCreateForm, PrEditForm,
};
