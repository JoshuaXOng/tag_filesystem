#[test]
fn printing_with_escaped_newlines_and_indents() {
    assert_eq!(
        "First part of line one \
        and second part of line one.",
        "First part of line one and second part of line one.");
}
