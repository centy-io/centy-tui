# Refactor forms.rs into DDD components

Break apart the monolithic `src/ui/forms.rs` file into separate, domain-driven components:

- Create separate form components per domain (IssueForm, PRForm, DocForm)
- Extract shared form field components (TextField, MultilineField, etc.)
- Apply Domain-Driven Design principles to organize code by domain context
- Consider creating a `src/ui/forms/` directory with:
  - `mod.rs` - exports and shared utilities
  - `fields.rs` - reusable form field components
  - `issue_form.rs` - issue create/edit forms
  - `pr_form.rs` - PR create/edit forms
  - `doc_form.rs` - doc create form
