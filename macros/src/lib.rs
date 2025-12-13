use crate::{backtrace::{_define_with_backtrace, _derive_backtrace}, tracing::instrument};

mod backtrace;
mod tracing;

#[proc_macro_attribute]
pub fn _instrument(
    attribute_arguments: proc_macro::TokenStream,
    function_definition: proc_macro::TokenStream)
-> proc_macro::TokenStream {
    instrument(attribute_arguments, function_definition)
}

#[proc_macro]
pub fn define_with_backtrace(code_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    _define_with_backtrace(code_tokens)
}

const BACKTRACE_MACRO_NAME: &str = "Backtrace";
#[proc_macro_derive(Backtrace, attributes(bt_from))]
pub fn derive_backtrace(code_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    _derive_backtrace(code_tokens)
}
