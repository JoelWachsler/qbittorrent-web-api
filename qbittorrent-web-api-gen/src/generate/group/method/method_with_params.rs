use std::rc::Rc;

use case::CaseExt;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    generate::util,
    parser::{self, ApiMethod, ApiParameters},
    types,
};

use super::{
    return_type::create_return_type, send_method_builder::SendMethodBuilder, MethodsAndExtra,
};

pub fn create_method_with_params(
    group: &parser::ApiGroup,
    method: &parser::ApiMethod,
    params: &parser::ApiParameters,
    method_name: &proc_macro2::Ident,
    url: &str,
) -> MethodsAndExtra {
    let param_type = util::to_ident(&format!(
        "{}{}Parameters",
        group.name.to_camel(),
        method.name.to_camel()
    ));

    let parameters = Parameters::new(params);
    let has_optional_parameters = !parameters.optional.is_empty();
    let generator = MethodGenerator {
        group,
        url,
        parameters,
        param_type,
        method,
        method_name,
    };

    if has_optional_parameters {
        generator.generate_method_with_builder()
    } else {
        generator.generate_method_without_builder()
    }
}

#[derive(Debug)]
struct MethodGenerator<'a> {
    group: &'a parser::ApiGroup,
    url: &'a str,
    parameters: Parameters<'a>,
    param_type: proc_macro2::Ident,
    method: &'a ApiMethod,
    method_name: &'a proc_macro2::Ident,
}

impl<'a> MethodGenerator<'a> {
    fn generate_method_without_builder(&self) -> MethodsAndExtra {
        let builder = SendMethodBuilder::new(self.method_name, self.url, quote! { self.auth })
            .description(&self.method.description)
            .with_args(&self.parameters.mandatory.generate_params());

        match create_return_type(self.group, self.method) {
            Some((return_type_name, return_type)) => {
                MethodsAndExtra::new(builder.return_type(&return_type_name).build())
                    .with_structs(return_type)
            }
            None => MethodsAndExtra::new(builder.build()),
        }
    }

    fn generate_method_with_builder(&self) -> MethodsAndExtra {
        let group_name = self.group_name();
        let send_builder = self.send_builder(&util::to_ident("send"));
        let send_impl_generator =
            SendImplGenerator::new(&group_name, &self.parameters, &self.param_type);

        let send = match create_return_type(self.group, self.method) {
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
        let builder = generate_builder(
            &self.parameters,
            self.method,
            self.method_name,
            &self.param_type,
        );
        let param_type = &self.param_type;

        let group_impl = quote! {
            pub struct #param_type<'a> {
                group: &'a #group_name<'a>,
                form: reqwest::multipart::Form,
            }

            #send
        };

        MethodsAndExtra::new(builder).with_structs(group_impl)
    }

    fn group_name(&self) -> proc_macro2::Ident {
        util::to_ident(&self.group.name.to_camel())
    }

    fn send_builder(&self, name: &proc_macro2::Ident) -> SendMethodBuilder {
        SendMethodBuilder::new(name, self.url, quote! { self.group.auth }).with_form()
    }
}

fn generate_builder(
    parameters: &Parameters,
    method: &ApiMethod,
    method_name: &proc_macro2::Ident,
    param_type: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let mandatory_param_names = parameters.mandatory.names();
    let mandatory_param_args = parameters.mandatory.generate_params();

    util::add_docs(
        &method.description,
        quote! {
            pub fn #method_name(&self, #(#mandatory_param_args),*) -> #param_type {
                #param_type::new(self, #(#mandatory_param_names),*)
            }
        },
    )
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
    params: Vec<Parameter<'a>>,
}

impl<'a> MandatoryParams<'a> {
    fn new(params: &'a ApiParameters) -> Self {
        Self {
            params: Parameter::from(&params.mandatory),
        }
    }

    fn generate_params(&self) -> Vec<TokenStream> {
        self.params
            .iter()
            .map(|p| p.generate_param_with_name())
            .collect()
    }

    fn form_builder(&self) -> Vec<TokenStream> {
        self.params
            .iter()
            .map(|param| {
                let name_ident = param.name_ident();
                let name = param.name();
                quote! { let form = form.text(#name, #name_ident.to_string()); }
            })
            .collect()
    }

    fn names(&self) -> Vec<TokenStream> {
        self.params
            .iter()
            .map(|p| p.name_ident())
            .map(|name_ident| quote! { #name_ident })
            .collect()
    }
}

#[derive(Debug)]
struct OptionalParams<'a> {
    params: Vec<Parameter<'a>>,
}

impl<'a> OptionalParams<'a> {
    fn new(params: &'a ApiParameters) -> Self {
        Self {
            params: Parameter::from(&params.optional),
        }
    }

    fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    fn generate_builder_methods(&self) -> Vec<TokenStream> {
        self.params
            .iter()
            .map(Self::generate_builder_method)
            .collect()
    }

    fn generate_builder_method(param: &Parameter) -> TokenStream {
        let name = param.name();
        let name_ident = param.name_ident();

        let param_type = util::to_ident(&param.p_type.to_borrowed_type());

        let builder_param = if param.p_type.should_borrow() {
            quote! { &#param_type }
        } else {
            quote! { #param_type }
        };

        util::add_docs(
            &param.p_type.get_type_info().description,
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

    fn from(parameters: &[types::Type]) -> Vec<Parameter<'_>> {
        parameters.iter().map(Parameter::new).collect()
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
