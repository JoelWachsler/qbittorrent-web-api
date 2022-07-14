use std::rc::Rc;

use case::CaseExt;
use quote::quote;

use crate::{
    generate::util,
    parser::{self, ApiParameters},
    types,
};

use super::{return_type::create_return_type, send_method_builder::SendMethodBuilder};

pub fn create_method_with_params(
    group: &parser::ApiGroup,
    method: &parser::ApiMethod,
    params: &parser::ApiParameters,
    method_name: &proc_macro2::Ident,
    url: &str,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let param_type = util::to_ident(&format!(
        "{}{}Parameters",
        group.name.to_camel(),
        method.name.to_camel()
    ));

    let rc_params = Rc::new(params);

    let mandatory_params = MandatoryParams::new(&rc_params);
    let optional_params = OptionalParams::new(&rc_params);

    let mandatory_param_args = mandatory_params.generate_params();

    let group_name = util::to_ident(&group.name.to_camel());
    let send_builder =
        SendMethodBuilder::new(&util::to_ident("send"), url, quote! { self.group.auth })
            .with_form();

    let generate_send_impl = |send_method: proc_macro2::TokenStream| {
        let optional_params = optional_params.generate_params();
        let mandatory_param_form_build = mandatory_params.param_builder();

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

    let mandatory_param_names = mandatory_params.names();

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

#[derive(Debug)]
struct MandatoryParams<'a> {
    params: &'a ApiParameters,
}

impl<'a> MandatoryParams<'a> {
    fn new(params: &'a ApiParameters) -> Self {
        Self { params }
    }

    fn generate_params(&self) -> Vec<proc_macro2::TokenStream> {
        self.params
            .mandatory
            .iter()
            .map(|p| p.to_parameter().generate_param_with_name())
            .collect()
    }

    fn param_builder(&self) -> Vec<proc_macro2::TokenStream> {
        self.params
            .mandatory
            .iter()
            .map(|p| p.to_parameter())
            .map(|param| {
                let name_ident = param.name_ident();
                let name = param.name();
                quote! { let form = form.text(#name, #name_ident.to_string()); }
            })
            .collect()
    }

    fn names(&self) -> Vec<proc_macro2::TokenStream> {
        self.params
            .mandatory
            .iter()
            .map(|p| p.to_parameter().name_ident())
            .map(|name_ident| quote! { #name_ident })
            .collect()
    }
}

#[derive(Debug)]
struct OptionalParams<'a> {
    params: &'a ApiParameters,
}

impl<'a> OptionalParams<'a> {
    fn new(params: &'a ApiParameters) -> Self {
        Self { params }
    }

    fn generate_params(&self) -> Vec<proc_macro2::TokenStream> {
        self.params
            .optional
            .iter()
            .map(Self::generate_param)
            .collect()
    }

    fn generate_param(param: &types::Type) -> proc_macro2::TokenStream {
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
}

#[derive(Debug)]
struct Parameter<'a> {
    p_type: &'a types::Type,
}

impl<'a> Parameter<'a> {
    fn new(p_type: &'a types::Type) -> Self {
        Self { p_type }
    }

    fn name(&self) -> String {
        self.p_type.get_type_info().name.to_snake()
    }

    fn name_ident(&self) -> proc_macro2::Ident {
        util::to_ident(&self.name())
    }

    fn generate_param_with_name(&self) -> proc_macro2::TokenStream {
        let t = util::to_ident(&self.p_type.to_borrowed_type());

        let name_ident = self.name_ident();
        let t = if self.p_type.should_borrow() {
            quote! { &#t }
        } else {
            quote! { #t }
        };

        quote! { #name_ident: #t }
    }
}

impl types::Type {
    fn to_parameter(&self) -> Parameter<'_> {
        Parameter::new(self)
    }
}
