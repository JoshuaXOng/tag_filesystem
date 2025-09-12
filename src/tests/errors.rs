use crate::{errors::{AnyError, Result_}, unwrap_or};

#[test]
fn executing_unwrap_or_on_result() {
    let result: Result_<_>  = Ok(1);
    let ok = unwrap_or!(result, e, {
        assert!(false);
        return;
    });
    assert_eq!(ok, 1);

    let result = Err(());
    let mut did_execute = false;
    unwrap_or!(result, e, {
        did_execute = true;
        assert_eq!(e, ());
    });
    assert!(did_execute);
}

#[test]
fn executing_unwrap_or_on_option() {
    let option = Some(1);
    let some = unwrap_or!(option, {
        assert!(false);
        return;
    });
    assert_eq!(some, 1);

    let option = None;
    let mut did_execute = false;
    unwrap_or!(option, {
        did_execute = true;
    });
    assert!(did_execute);
}
