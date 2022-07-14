use std::rc::Rc;

use case::CaseExt;
use proc_macro2::TokenStream;
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
) -> (TokenStream, Option<TokenStream>) {
    let param_type = util::to_ident(&format!(
        "{}{}Parameters",
        group.name.to_camel(),
        method.name.to_camel()
    ));

    let parameters = Parameters::new(params);

    let mandatory_param_args = parameters.mandatory.generate_params();

    let group_name = util::to_ident(&group.name.to_camel());
    let send_builder =
        SendMethodBuilder::new(&util::to_ident("send"), url, quote! { self.group.auth })
            .with_form();

    let send_impl_generator = SendImplGenerator::new(&group_name, &parameters, &param_type);

    let send = match create_return_type(group, method) {
        Some((return_type_name, return_type)) => {
            let send_impl =
                send_impl_generator.generate(send_builder.return_type(&return_type_name));

            quote! {
                #send_impl
                #return_type
            }
        }
        None => send_impl_generator.generate(send_builder),
    };

    let mandatory_param_names = parameters.mandatory.names();

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
struct SendImplGenerator<'a> {
    group_name: &'a proc_macro2::Ident,
    parameters: &'a Parameters<'a>,
    param_type: &'a proc_macro2::Ident,
}

impl<'a> SendImplGenerator<'a> {
    fn new(
        group_name: &'a proc_macro2::Ident,
        parameters: &'a Parameters<'a>,
        param_type: &'a proc_macro2::Ident,
    ) -> Self {
        Self {
            group_name,
            parameters,
            param_type,
        }
    }

    fn generate(&self, send_method_builder: SendMethodBuilder) -> TokenStream {
        let parameters = self.parameters;

        let optional_builder_methods = parameters.optional.generate_builder_methods();
        let mandatory_param_form_build = parameters.mandatory.form_builder();
        let mandatory_param_args = parameters.mandatory.generate_params();
        let param_type = self.param_type;
        let group_name = self.group_name;
        let send_method = send_method_builder.build();

        quote! {
            impl<'a> #param_type<'a> {
                fn new(group: &'a #group_name, #(#mandatory_param_args),*) -> Self {
                    let form = reqwest::multipart::Form::new();
                    #(#mandatory_param_form_build)*
                    Self { group, form }
                }

                #(#optional_builder_methods)*
                #send_method
            }
        }
    }
}

#[derive(Debug)]
struct Parameters<'a> {
    mandatory: MandatoryParams<'a>,
    optional: OptionalParams<'a>,
}

impl<'a> Parameters<'a> {
    fn new(api_parameters: &'a ApiParameters) -> Self {
        let rc_params = Rc::new(api_parameters);

        let mandatory = MandatoryParams::new(&rc_params);
        let optional = OptionalParams::new(&rc_params);

        Self {
            mandatory,
            optional,
        }
    }
}

#[derive(Debug)]
struct MandatoryParams<'a> {
    params: &'a ApiParameters,
}

impl<'a> MandatoryParams<'a> {
    fn new(params: &'a ApiParameters) -> Self {
        Self { params }
    }

    fn generate_params(&self) -> Vec<TokenStream> {
        self.params
            .mandatory
            .iter()
            .map(|p| p.to_parameter().generate_param_with_name())
            .collect()
    }

    fn form_builder(&self) -> Vec<TokenStream> {
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

    fn names(&self) -> Vec<TokenStream> {
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

    fn generate_builder_methods(&self) -> Vec<TokenStream> {
        self.params
            .optional
            .iter()
            .map(Self::generate_builder_method)
            .collect()
    }

    fn generate_builder_method(param: &types::Type) -> TokenStream {
        let parameter = param.to_parameter();
        let name = parameter.name();
        let name_ident = parameter.name_ident();

        let param_type = util::to_ident(&param.to_borrowed_type());

        let builder_param = if param.should_borrow() {
            quote! { &#param_type }
        } else {
            quote! { #param_type }
        };

        util::add_docs(
            &param.get_type_info().description,
            quote! {
                pub fn #name_ident(mut self, value: #builder_param) -> Self {
                    self.form = self.form.text(#name, value.to_string());
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

    fn generate_param_with_name(&self) -> TokenStream {
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
