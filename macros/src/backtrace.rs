use proc_macro2::{Punct, Spacing, Span};
use syn::{parse_macro_input, punctuated::Punctuated, AttrStyle, Ident, Item,
    Meta, Path, Token, Data, DeriveInput};
use quote::quote;

use crate::BACKTRACE_MACRO_NAME;

macro_rules! if_not_variant {
    ($enum: expr, $the_variant: path, $other_variant: ident, $else_do: expr) => {
        {
            match $enum {
                $the_variant(variant) => variant,
                $other_variant => _ = $else_do
            }
        }
    };
}

pub fn _define_with_backtrace(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let dyn_codes = get_into_dyn_codes(
        Some(Punct::new('$', Spacing::Alone)),
        Ident::new("error_type", Span::call_site()));
    quote! {
        #[derive(Debug)]
        pub struct WithBacktrace<E> {
            pub error: E,
            pub backtrace: std::backtrace::Backtrace
        }

        impl<E> WithBacktrace<E> {
            pub fn new(error: E) -> Self {
                Self {
                    error,
                    backtrace: std::backtrace::Backtrace::capture()
                }
            }

            pub fn get(&self) -> &E {
                &self.error
            }

            pub fn get_owned(self) -> E {
                self.error
            }
        }

        impl<E> std::ops::Deref for WithBacktrace<E> {
            type Target = E;

            fn deref(&self) -> &Self::Target {
                self.get()
            }
        }

        impl<E> From<E> for WithBacktrace<E> {
            fn from(error: E) -> Self {
                Self::new(error)
            }
        }

        #[allow(dead_code)]
        macro_rules! define_to_dyn {
            ($error_type: ty) => {
                #dyn_codes
            };
        }
    }.into()
}

const BACKTRACE_FROM_HELPER: &str = "bt_from";

pub fn _derive_backtrace(code_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_item = parse_macro_input!(code_tokens as DeriveInput);
    if let Data::Union(_) = derive_item.data {
        return syn::Error::new_spanned(
            derive_item.ident,
            format!("{BACKTRACE_MACRO_NAME} cannot be applied to unions."))
            .to_compile_error()
            .into();
    }
    let to_identifier = derive_item.ident;
    let attributes = derive_item.attrs;

    let mut to_codes = vec![];

    for attribute in attributes {
        if AttrStyle::Outer != attribute.style {
            continue;
        }
        let helper_attributes = if_not_variant!(attribute.meta, Meta::List, _other, {
            continue;
        }); 
        let attribute_identifier = if_not_variant!(helper_attributes.path.get_ident(),
            Some, _option, continue);
        if attribute_identifier.to_string() != BACKTRACE_FROM_HELPER {
            continue;
        }

        let helper_arguments: proc_macro::TokenStream = helper_attributes.tokens.into();
        let arguments_parser = Punctuated::<Path, Token![,]>::parse_terminated;
        let helper_arguments = parse_macro_input!(helper_arguments with arguments_parser);

        for from_identifier in helper_arguments {
            to_codes.push(quote! {
                impl From<crate::WithBacktrace<#from_identifier>> for crate::WithBacktrace<#to_identifier> {
                    fn from(value: crate::WithBacktrace<#from_identifier>) -> Self {
                        Self {
                            error: #to_identifier::from(value.error),
                            backtrace: value.backtrace,
                        } 
                    }
                }

                impl From<#from_identifier> for crate::WithBacktrace<#to_identifier> {
                    fn from(value: #from_identifier) -> Self {
                        #to_identifier::from(value).into()
                    }
                }
            }); 
        }
    }

    let dyn_codes = get_into_dyn_codes(None, to_identifier);

    quote! {
        #(#to_codes)*
        #dyn_codes
    }.into()
}

fn get_into_dyn_codes(prefix_punctuation: Option<Punct>, to_identifier: Ident) -> proc_macro2::TokenStream {
    quote! {
        impl From<#prefix_punctuation #to_identifier> for crate::WithBacktrace<Box<dyn std::error::Error>> {
            fn from(value: #prefix_punctuation #to_identifier) -> Self {
                Self {
                    error: value.into(),
                    backtrace: std::backtrace::Backtrace::capture()
                }
            }
        }

        impl From<crate::WithBacktrace<#prefix_punctuation #to_identifier>> for crate::WithBacktrace<Box<dyn std::error::Error>> {
            fn from(value: crate::WithBacktrace<#prefix_punctuation #to_identifier>) -> Self {
                Self {
                    error: value.error.into(),
                    backtrace: value.backtrace,
                } 
            }
        }

        impl From<#prefix_punctuation #to_identifier> for crate::WithBacktrace<Box<dyn std::error::Error + Send + Sync>> {
            fn from(value: #prefix_punctuation #to_identifier) -> Self {
                Self {
                    error: value.into(),
                    backtrace: std::backtrace::Backtrace::capture()
                }
            }
        }

        impl From<crate::WithBacktrace<#prefix_punctuation #to_identifier>> for crate::WithBacktrace<Box<dyn std::error::Error + Send + Sync>> {
            fn from(value: crate::WithBacktrace<#prefix_punctuation #to_identifier>) -> Self {
                Self {
                    error: value.error.into(),
                    backtrace: value.backtrace,
                } 
            }
        }
    }
}
