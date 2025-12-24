use crate::{errors::{AnyError, ResultBtAny}, return_errors, unwrap_or, WithBacktrace};

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

// TODO: You can return `Result` from tests now? Convert.
#[test]
fn running_return_errors() {
    let e = running_return_errors_with_errrors_as_fraction().unwrap_err();
    assert_eq!(e.to_string(), 
        "Not all checks passed. Error 1. Error 3.");

    let e = running_return_errors_with_all_errrors().unwrap_err();
    assert_eq!(e.to_string(), 
        "Not all checks passed. Error 1. Error 2. Error 3.");

    let mut string = String::new();
    running_return_errors_with_no_errrors(&mut string).unwrap();
    assert_eq!(string, "Ok 1. Ok 2. Ok 3.");
}

fn running_return_errors_with_errrors_as_fraction() -> ResultBtAny<()> {
    let result_1 = Err::<u32, _>("Error 1.");
    let result_2 = Ok::<u32, &'static str>(67);
    let result_3 = Err::<u32, _>("Error 3.");
    return_errors!("Not all checks passed.", result_1, result_2, result_3);
    Ok(())
}

fn running_return_errors_with_all_errrors() -> ResultBtAny<()> {
    let result_1 = Err::<u32, _>("Error 1.");
    let result_2 = Err::<u32, _>("Error 2.");
    let result_3 = Err::<u32, _>("Error 3.");
    return_errors!("Not all checks passed.", result_1, result_2, result_3);
    Ok(())
}

fn running_return_errors_with_no_errrors(string: &mut String) -> ResultBtAny<()> {
    let result_1 = Ok::<_, WithBacktrace<AnyError>>("Ok 1.");
    let result_2 = Ok::<_, WithBacktrace<AnyError>>("Ok 2.");
    let result_3 = Ok::<_, WithBacktrace<AnyError>>("Ok 3.");
    return_errors!("Not all checks passed.", result_1, result_2, result_3);
    string.push_str(result_1);
    string.push_str(" ");
    string.push_str(result_2);
    string.push_str(" ");
    string.push_str(result_3);
    Ok(())
}
