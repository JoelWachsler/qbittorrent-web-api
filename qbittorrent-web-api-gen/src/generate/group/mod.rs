mod method;

use crate::parser;
use case::CaseExt;
use quote::quote;

use self::method::generate_methods;

use super::{skeleton::auth_ident, util};

pub fn generate_groups(groups: Vec<parser::ApiGroup>) -> proc_macro2::TokenStream {
    let gr = groups
        .iter()
        // implemented manually
        .filter(|group| group.name != "authentication")
        .map(generate_group);

    quote! {
        #(#gr)*
    }
}

fn generate_group(group: &parser::ApiGroup) -> proc_macro2::TokenStream {
    let group_name_camel = util::to_ident(&group.name.to_camel());
    let group_name_snake = util::to_ident(&group.name.to_snake());
    let auth = auth_ident();
    let methods = generate_methods(group, &auth, &group_name_camel);

    let group_method = util::add_docs(
        &group.description,
        quote! {
            pub fn #group_name_snake(&self) -> #group_name_camel {
                #group_name_camel::new(self)
            }
        },
    );

    quote! {
        pub struct #group_name_camel<'a> {
            auth: &'a #auth,
        }

        #methods

        impl #auth {
            #group_method
        }
    }
}
