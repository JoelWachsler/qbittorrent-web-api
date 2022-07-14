use quote::quote;

use super::{
    return_type::create_return_type, send_method_builder::SendMethodBuilder, MethodsAndExtra,
};
use crate::parser;

pub fn create_method_without_params(
    group: &parser::ApiGroup,
    method: &parser::ApiMethod,
    method_name: proc_macro2::Ident,
    url: &str,
) -> MethodsAndExtra {
    let builder = SendMethodBuilder::new(&method_name, url, quote! { self.auth })
        .description(&method.description);

    match create_return_type(group, method) {
        Some((return_type_name, return_type)) => {
            MethodsAndExtra::new(builder.return_type(&return_type_name).build())
                .with_structs(return_type)
        }
        None => {
            // assume that all methods without a return type returns a string
            MethodsAndExtra::new(builder.build())
        }
    }
}
