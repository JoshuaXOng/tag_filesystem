use crate::tracing::instrument;

mod tracing;

#[proc_macro_attribute]
pub fn _instrument(
    attribute_arguments: proc_macro::TokenStream,
    function_definition: proc_macro::TokenStream)
-> proc_macro::TokenStream {
    instrument(attribute_arguments, function_definition)
}
