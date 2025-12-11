use syn::{parse_macro_input, punctuated::Punctuated, AttrStyle, Ident, Item, Meta,
    MetaList, Token};
use quote::quote;

// TODO: How to reduce nesting w/ macros 

pub fn _define_with_backtrace(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote! {
        #[derive(Debug)]
        struct WithBacktrace<E> {
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

        macro_rules! define_to_dyn {
            ($error_type: ty) => {
                impl From<$error_type> for WithBacktrace<Box<dyn std::error::Error>> {
                    fn from(value: $error_type) -> Self {
                        Self {
                            error: value.into(),
                            backtrace: std::backtrace::Backtrace::capture()
                        }
                    }
                }

                impl From<WithBacktrace<$error_type>> for WithBacktrace<Box<dyn std::error::Error>> {
                    fn from(value: WithBacktrace<$error_type>) -> Self {
                        Self {
                            error: value.error.into(),
                            backtrace: value.backtrace,
                        } 
                    }
                }
            };
        }
    }.into()
}

pub fn _derive_backtrace(code_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(code_tokens as Item);
    let (to_identifier, attributes) = match item {
        Item::Enum(_enum) => (_enum.ident, _enum.attrs),
        Item::Struct(_struct) => (_struct.ident, _struct.attrs),
        _ => panic!("Enum or s only TODO: better warnings")
    };

    let mut to_codes = vec![];

    for attribute in attributes {
        if AttrStyle::Outer != attribute.style {
            continue;
        }
        if let Meta::List(MetaList { path, tokens, .. }) = attribute.meta {
            let tokens: proc_macro::TokenStream = tokens.into();

            if let Some(attribute_identifier) = path.get_ident() {
                // TODO: Assign to v. ident.
                if attribute_identifier.to_string() != "bt_from" {
                    continue;
                }

                let arguments_parser = Punctuated::<Ident, Token![,]>::parse_terminated;
                let attribute_arguments = parse_macro_input!(tokens with arguments_parser);

                for from_identifier in attribute_arguments {
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
        }
    }

    // TODO: Deduplicate
    // E.g., something like `crate::define_to_dyn!(#to_identifier)`
    let dyn_codes = quote! {
        impl From<#to_identifier> for crate::WithBacktrace<Box<dyn std::error::Error>> {
            fn from(value: #to_identifier) -> Self {
                Self {
                    error: value.into(),
                    backtrace: std::backtrace::Backtrace::capture()
                }
            }
        }

        impl From<crate::WithBacktrace<#to_identifier>> for crate::WithBacktrace<Box<dyn std::error::Error>> {
            fn from(value: crate::WithBacktrace<#to_identifier>) -> Self {
                Self {
                    error: value.error.into(),
                    backtrace: value.backtrace,
                } 
            }
        }
    };

    quote! {
        #(#to_codes)*
        #dyn_codes
    }.into()
}
