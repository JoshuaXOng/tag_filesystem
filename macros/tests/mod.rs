#![cfg_attr(test, allow(unused))]

use std::io::{self, Write};

use drums::_instrument;
use tracing::{info, instrument, Level};
use tracing_subscriber::{self, fmt::MakeWriter};
use tracing_test::traced_test;

#[test]
#[traced_test]
fn calling_instrumented_function_that_traces() {
    function_0(1, 2, 3);
    assert!(logs_contain("INFO calling_instrumented_function_that_traces:function_0: r#mod: hello there :>"));
    function_1(4, 5, 6);
    assert!(logs_contain("INFO calling_instrumented_function_that_traces:function_1{a=4}: r#mod: hello there :>"));
    function_2(7, 8, 9);
    assert!(logs_contain("INFO calling_instrumented_function_that_traces:function_2{a=7}: r#mod: hello there :>"));
    function_3(10, 11, 12);
    assert!(logs_contain("INFO calling_instrumented_function_that_traces:function_3{a=10}: r#mod: hello there :>"));
}

#[instrument(skip_all, fields(a))]
fn function_0(a: i32, b: i32, c: i32) {
    info!("hello there :>");
}

#[_instrument(skip_all, fields(a = a))]
fn function_1(a: i32, b: i32, c: i32) {
    info!("hello there :>");
}

#[_instrument(skip_all, fields(a))]
fn function_2(a: i32, b: i32, c: i32) {
    info!("hello there :>");
}

#[_instrument(skip_all, fields(a = a))]
fn function_3(a: i32, b: i32, c: i32) {
    info!("hello there :>");
}
