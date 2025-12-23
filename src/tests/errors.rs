use crate::{coalesce, errors::{AnyError, ResultBtAny}, unwrap_or};

#[test]
fn executing_unwrap_or_on_result() {
    let result: ResultBtAny<_>  = Ok(1);
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
//
// TODO: You can return `Result` from tests now? Convert.
#[test]
fn running_coalesce() {
    let result_1 = Err::<u32, _>("Could not find the wrench.");
    let result_2 = Ok::<u32, &'static str>(67);
    let result_3 = Err::<u32, _>("Salt grains were too big for the salt shaker.");
    let e = coalesce!("Not all checks passed.", result_1, result_2, result_3)
        .unwrap_err();
    assert_eq!(e.to_string(), 
        "Not all checks passed. Salt grains were too big for the salt shaker. \
        Could not find the wrench.");
}
