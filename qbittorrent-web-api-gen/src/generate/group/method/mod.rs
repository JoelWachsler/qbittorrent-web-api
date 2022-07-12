mod method_builder;
mod method_with_params;
mod method_without_params;
mod return_type;

use crate::{generate::util, parser};
use case::CaseExt;
use quote::quote;

use self::{
    method_with_params::create_method_with_params,
    method_without_params::create_method_without_params,
};

pub fn generate_methods(
    group: &parser::ApiGroup,
    auth: &syn::Ident,
    group_name_camel: &syn::Ident,
) -> proc_macro2::TokenStream {
    let methods_and_param_structs = group
        .methods
        .iter()
        .map(|method| generate_method(group, method));

    let methods = methods_and_param_structs.clone().map(|(method, ..)| method);
    let structs = methods_and_param_structs.flat_map(|(_, s)| s);

    quote! {
        impl <'a> #group_name_camel<'a> {
            pub fn new(auth: &'a #auth) -> Self {
                Self { auth }
            }

            #(#methods)*
        }

        #(#structs)*
    }
}

fn generate_method(
    group: &parser::ApiGroup,
    method: &parser::ApiMethod,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let method_name = util::to_ident(&method.name.to_snake());
    let url = format!("/api/v2/{}/{}", group.url, method.url);

    match &method.parameters {
        Some(params) => create_method_with_params(group, method, params, &method_name, &url),
        None => create_method_without_params(group, method, method_name, &url),
    }
}
