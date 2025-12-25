//! Form domain layer

mod field;
mod form_state;

pub use field::FormField;
pub use form_state::{
    DocCreateForm, Form, FormState, IssueCreateForm, IssueEditForm, PrCreateForm, PrEditForm,
};
