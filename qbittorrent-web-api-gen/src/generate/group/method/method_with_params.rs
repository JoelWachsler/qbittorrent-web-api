use case::CaseExt;
use quote::quote;

use crate::{generate::util, parser, types};

use super::{method_builder::MethodBuilder, return_type::create_return_type};

pub fn create_method_with_params(
    group: &parser::ApiGroup,
    method: &parser::ApiMethod,
    params: &[types::Type],
    method_name: &proc_macro2::Ident,
    url: &str,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let parameter_type = util::to_ident(&format!(
        "{}{}Parameters",
        group.name.to_camel(),
        method.name.to_camel()
    ));

    let mandatory_params = params
        .iter()
        .filter(|param| !param.get_type_info().is_optional);

    let mandatory_param_args = mandatory_params.clone().map(|param| {
        let name = util::to_ident(&param.get_type_info().name.to_snake());
        let t = util::to_ident(&param.to_borrowed_type());

        if param.should_borrow() {
            quote! {
                #name: &#t
            }
        } else {
            quote! {
                #name: #t
            }
        }
    });

    let mandatory_param_names = mandatory_params.clone().map(|param| {
        let name = util::to_ident(&param.get_type_info().name.to_snake());

        quote! {
            #name
        }
    });

    let mandatory_param_args_clone = mandatory_param_args.clone();
    let mandatory_param_form_build = mandatory_params.map(|param| {
        let n = &param.get_type_info().name;
        let name = util::to_ident(&n.to_snake());

        quote! {
            let form = form.text(#n, #name.to_string());
        }
    });

    let optional_params = params
        .iter()
        .filter(|param| param.get_type_info().is_optional)
        .map(|param| {
            let n = &param.get_type_info().name;
            let name = util::to_ident(&n.to_snake());
            let t = util::to_ident(&param.to_borrowed_type());

            let builder_param = if param.should_borrow() {
                quote! { &#t }
            } else {
                quote! { #t }
            };

            util::add_docs(
                &param.get_type_info().description,
                quote! {
                    pub fn #name(mut self, value: #builder_param) -> Self {
                        self.form = self.form.text(#n, value.to_string());
                        self
                    }
                },
            )
        });

    let group_name = util::to_ident(&group.name.to_camel());
    let send_builder =
        MethodBuilder::new(&util::to_ident("send"), url, quote! { self.group.auth }).with_form();

    let send_new_method = quote! {
        fn new(group: &'a #group_name, #(#mandatory_param_args),*) -> Self {
            let form = reqwest::multipart::Form::new();
            #(#mandatory_param_form_build)*
            Self { group, form }
        }
    };

    let send = match create_return_type(group, method) {
        Some((return_type_name, return_type)) => {
            let send_method = send_builder.return_type(&return_type_name).build();

            quote! {
                impl<'a> #parameter_type<'a> {
                    #send_new_method
                    #(#optional_params)*
                    #send_method
                }

                #return_type
            }
        }
        None => {
            let send_method = send_builder.build();

            quote! {
                impl<'a> #parameter_type<'a> {
                    #send_new_method
                    #(#optional_params)*
                    #send_method
                }
            }
        }
    };

    (
        util::add_docs(
            &method.description,
            quote! {
                pub fn #method_name(&self, #(#mandatory_param_args_clone),*) -> #parameter_type {
                    #parameter_type::new(self, #(#mandatory_param_names),*)
                }
            },
        ),
        Some(quote! {
            pub struct #parameter_type<'a> {
                group: &'a #group_name<'a>,
                form: reqwest::multipart::Form,
            }

            #send
        }),
    )
}
