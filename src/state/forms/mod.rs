//! Form domain layer
//!
//! This module contains form-related domain entities and logic:
//! - `FormField`: Value object representing a single form field
//! - `FormState`: Aggregate enum for all form types
//! - Individual form structs for each entity type

mod field;
mod form_state;

pub use field::{FieldValue, FormField};
pub use form_state::{
    DocCreateForm, Form, FormState, IssueCreateForm, IssueEditForm, PrCreateForm, PrEditForm,
};
