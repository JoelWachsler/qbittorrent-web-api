use case::CaseExt;
use quote::quote;

use crate::{generate::util, parser, types};

use super::{return_type::create_return_type, send_method_builder::SendMethodBuilder};

pub fn create_method_with_params(
    group: &parser::ApiGroup,
    method: &parser::ApiMethod,
    params: &[types::Type],
    method_name: &proc_macro2::Ident,
    url: &str,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let param_type = util::to_ident(&format!(
        "{}{}Parameters",
        group.name.to_camel(),
        method.name.to_camel()
    ));

    let mandatory_params = mandatory_params(params);
    let mandatory_param_args = generate_mandatory_params(&mandatory_params);

    let mandatory_param_names = mandatory_params.iter().map(|param| {
        let (name, ..) = param_name(param);
        quote! { #name }
    });

    let group_name = util::to_ident(&group.name.to_camel());
    let send_builder =
        SendMethodBuilder::new(&util::to_ident("send"), url, quote! { self.group.auth })
            .with_form();

    let generate_send_impl = |send_method: proc_macro2::TokenStream| {
        let optional_params = generate_optional_params(params);
        let mandatory_param_form_build = generate_mandatory_param_builder(&mandatory_params);

        quote! {
            impl<'a> #param_type<'a> {
                fn new(group: &'a #group_name, #(#mandatory_param_args),*) -> Self {
                    let form = reqwest::multipart::Form::new();
                    #(#mandatory_param_form_build)*
                    Self { group, form }
                }

                #(#optional_params)*
                #send_method
            }
        }
    };

    let send = match create_return_type(group, method) {
        Some((return_type_name, return_type)) => {
            let send_impl = generate_send_impl(send_builder.return_type(&return_type_name).build());

            quote! {
                #send_impl
                #return_type
            }
        }
        None => generate_send_impl(send_builder.build()),
    };

    let builder = util::add_docs(
        &method.description,
        quote! {
            pub fn #method_name(&self, #(#mandatory_param_args),*) -> #param_type {
                #param_type::new(self, #(#mandatory_param_names),*)
            }
        },
    );

    let group_impl = quote! {
        pub struct #param_type<'a> {
            group: &'a #group_name<'a>,
            form: reqwest::multipart::Form,
        }

        #send
    };

    (builder, Some(group_impl))
}

fn generate_mandatory_params(mandatory_params: &[&types::Type]) -> Vec<proc_macro2::TokenStream> {
    mandatory_params
        .iter()
        .map(|param| param_with_name(param))
        .collect()
}

fn generate_mandatory_param_builder(
    mandatory_params: &[&types::Type],
) -> Vec<proc_macro2::TokenStream> {
    mandatory_params
        .iter()
        .map(|param| {
            let (name, name_as_str) = param_name(param);
            quote! { let form = form.text(#name_as_str, #name.to_string()); }
        })
        .collect()
}

fn generate_optional_params(params: &[types::Type]) -> Vec<proc_macro2::TokenStream> {
    params
        .iter()
        .filter(|param| param.get_type_info().is_optional)
        .map(generate_optional_param)
        .collect()
}

fn mandatory_params(params: &[types::Type]) -> Vec<&types::Type> {
    params
        .iter()
        .filter(|param| !param.get_type_info().is_optional)
        .collect()
}

fn generate_optional_param(param: &types::Type) -> proc_macro2::TokenStream {
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
}

fn param_name(param: &types::Type) -> (proc_macro2::Ident, String) {
    let name_as_str = param.get_type_info().name.to_snake();
    (util::to_ident(&name_as_str), name_as_str)
}

fn param_with_name(param: &types::Type) -> proc_macro2::TokenStream {
    let t = util::to_ident(&param.to_borrowed_type());

    let (name, ..) = param_name(param);
    let t = if param.should_borrow() {
        quote! { &#t }
    } else {
        quote! { #t }
    };

    quote! { #name: #t }
}
