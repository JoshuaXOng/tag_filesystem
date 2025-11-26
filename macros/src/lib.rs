use syn::{parse_macro_input, parse_quote, Expr, ExprAssign, ExprCall, ExprPath, ItemFn, punctuated::Punctuated, Token};
use quote::{quote, ToTokens};

#[proc_macro_attribute]
pub fn _instrument(
    attribute_arguments: proc_macro::TokenStream,
    function_definition: proc_macro::TokenStream)
-> proc_macro::TokenStream {
    let function_definition = parse_macro_input!(function_definition as ItemFn);

    let remaining_attributes = &function_definition.attrs;
    let function_visability = &function_definition.vis;
    let function_signature = &function_definition.sig;
    let code_block = &function_definition.block;

    let arguments_parser = Punctuated::<Expr, Token![,]>::parse_terminated;
    let mut attribute_arguments = parse_macro_input!(attribute_arguments with arguments_parser);
    for attribute_argument in attribute_arguments.iter_mut() {
        if let Expr::Call(ExprCall { func, args, .. }) = attribute_argument {
            if let Expr::Path(ExprPath { ref path, .. }) = **func {
                if let Some(function_name) = path.segments.get(0) {
                    if function_name.ident.to_string() == "fields" {
                        for fields_argument in &mut *args {
                            if let Expr::Path(_) = &fields_argument {
                                *fields_argument = Expr::Assign(ExprAssign {
                                    attrs: vec![],
                                    left: Box::new(fields_argument.clone()),
                                    eq_token: parse_quote! { = },
                                    right: Box::new(fields_argument.clone())
                                });
                            };
                        }
                    }
                }
            }
        }
    }

    let attribute_arguments = proc_macro2::TokenStream::from(attribute_arguments.to_token_stream());

    quote! {
        #[tracing::instrument(#attribute_arguments)]
        #(#remaining_attributes)*
        #function_visability #function_signature #code_block
    }.into()
}
