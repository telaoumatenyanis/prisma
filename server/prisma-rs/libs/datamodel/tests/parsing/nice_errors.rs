use crate::common::*;
use datamodel::ast::Span;
use datamodel::errors::ValidationError;

#[test]
fn nice_error_for_missing_model_keyword() {
    let dml = r#"
    User {
        id Int @id
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(ValidationError::new_parser_error(
        &vec![
            "end of input",
            "type declaration",
            "model declaration",
            "enum declaration",
            "source definition",
            "generator definition",
        ],
        Span::new(5, 5),
    ));
}
#[test]
fn nice_error_for_missing_model_keyword_2() {
    let dml = r#"
    model User {
        id Int @id
    }
    Todo {
        id Int @id
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(ValidationError::new_parser_error(
        &vec![
            "end of input",
            "type declaration",
            "model declaration",
            "enum declaration",
            "source definition",
            "generator definition",
        ],
        Span::new(47, 47),
    ));
}

#[test]
fn nice_error_on_incorrect_enum_field() {
    let dml = r#"
    enum Role {
        A-dmin
        User
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(ValidationError::new_parser_error(
        &vec!["End of block (\"}\")", "enum field declaration"],
        Span::new(26, 26),
    ));
}

#[test]
fn nice_error_missing_type() {
    let dml = r#"
    model User {
        id Int @id
        name
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(ValidationError::new_parser_error(
        &vec!["field type"],
        Span::new(54, 54),
    ));
}

#[test]
fn nice_error_missing_directive_name() {
    let dml = r#"
    model User {
        id Int @id @
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(ValidationError::new_parser_error(
        &vec!["directive"],
        Span::new(43, 43),
    ));
}

// TODO: This case is not nice because the "{ }" belong to the declaration.
#[test]
fn nice_error_missing_braces() {
    let dml = r#"
    model User 
        id Int @id
    "#;

    let error = parse_error(dml);

    error.assert_is(ValidationError::new_parser_error(
        &vec!["Start of block (\"{\")"],
        Span::new(25, 25),
    ));
}

#[test]
fn nice_error_broken_field_type_legacy_list() {
    let dml = r#"
    model User {
        id [Int] @id
    }"#;

    let error = parse_error(dml);

    error.assert_is(ValidationError::new_legacy_parser_error(
        "To specify a list, please use `Type[]` instead of `[Type]`.",
        Span::new(29, 34),
    ));
}

#[test]
fn nice_error_broken_field_type_legacy_colon() {
    let dml = r#"
    model User {
        id: Int @id
    }"#;

    let error = parse_error(dml);

    error.assert_is(ValidationError::new_legacy_parser_error(
        "Field declarations don't require a `:`.",
        Span::new(28, 29),
    ));
}

#[test]
fn nice_error_broken_field_type_legacy_required() {
    let dml = r#"
    model User {
        id Int! @id
    }"#;

    let error = parse_error(dml);

    error.assert_is(ValidationError::new_legacy_parser_error(
        "Fields are required by default, `!` is no longer required.",
        Span::new(29, 33),
    ));
}

#[test]
fn nice_error_legacy_model_decl() {
    let dml = r#"
    type User {
        id Int @id
    }"#;

    let error = parse_error(dml);

    error.assert_is(ValidationError::new_legacy_parser_error(
        "Model declarations have to be indicated with the `model` keyword.",
        Span::new(5, 9),
    ));
}
