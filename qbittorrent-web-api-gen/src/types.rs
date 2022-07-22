use std::collections::HashMap;

use case::CaseExt;
use regex::RegexBuilder;

#[derive(Debug, Clone)]
pub struct TypeDescriptions {
    pub value: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct TypeDescription {
    pub values: Vec<TypeDescriptions>,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub is_optional: bool,
    is_list: bool,
    pub description: Option<String>,
    pub type_description: Option<TypeDescription>,
}

impl TypeInfo {
    pub fn new(
        name: &str,
        is_optional: bool,
        is_list: bool,
        description: Option<String>,
        type_description: Option<TypeDescription>,
    ) -> Self {
        Self {
            name: name.into(),
            is_optional,
            is_list,
            description,
            type_description,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    pub type_info: TypeInfo,
    pub ref_type: String,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub type_info: TypeInfo,
    pub values: Vec<EnumValue>,
}

#[derive(Debug, Clone)]
pub struct EnumValue {
    pub description: Option<String>,
    pub key: String,
    pub value: String,
}

pub const OPTIONAL: &str = "_optional_";

#[derive(Debug, Clone)]
pub enum Type {
    Number(TypeInfo),
    Float(TypeInfo),
    Bool(TypeInfo),
    String(TypeInfo),
    StringArray(TypeInfo),
    Object(Object),
}

impl Type {
    pub fn to_owned_type(&self) -> String {
        match self {
            Type::Number(_) => "i128".into(),
            Type::Float(_) => "f32".into(),
            Type::Bool(_) => "bool".into(),
            Type::String(_) => "String".into(),
            Type::StringArray(_) => "String".into(),
            Type::Object(_) => "String".into(),
        }
    }

    pub fn is_list(&self) -> bool {
        self.get_type_info().is_list || matches!(self, Type::StringArray(_))
    }

    pub fn to_borrowed_type(&self) -> String {
        match self {
            Type::Number(_) => "i32".into(),
            Type::Float(_) => "f32".into(),
            Type::Bool(_) => "bool".into(),
            Type::String(_) => "str".into(),
            Type::StringArray(_) => "&[str]".into(),
            Type::Object(_) => "str".into(),
        }
    }

    pub fn should_borrow(&self) -> bool {
        matches!(self, Type::String(_) | Type::Object(_))
    }

    pub fn is_optional(&self) -> bool {
        self.get_type_info().is_optional
    }

    pub fn get_type_info(&self) -> &TypeInfo {
        match self {
            Type::Number(t) => t,
            Type::Float(t) => t,
            Type::Bool(t) => t,
            Type::String(t) => t,
            Type::StringArray(t) => t,
            Type::Object(t) => &t.type_info,
        }
    }

    pub fn from(
        type_as_str: &str,
        name: &str,
        description: Option<String>,
        types: &HashMap<String, TypeDescription>,
    ) -> Option<Type> {
        let available_types = types.get(name).cloned();
        let type_name = match name.split_once(OPTIONAL) {
            Some((split, _)) => split,
            None => name,
        }
        .trim();

        let is_optional = name.contains(OPTIONAL);
        let create_type_info = || {
            TypeInfo::new(
                type_name,
                is_optional,
                false,
                description.clone(),
                available_types.clone(),
            )
        };

        let create_object_type = |name: &str| {
            Some(Type::Object(Object {
                type_info: create_type_info(),
                ref_type: name.to_camel(),
            }))
        };

        match type_as_str {
            "bool" => Some(Type::Bool(create_type_info())),
            "integer" | "number" | "int" => Some(Type::Number(create_type_info())),
            "string" => Some(Type::String(create_type_info())),
            "array" => description
                .extract_type()
                .and_then(|t| create_object_type(&t))
                .or_else(|| Some(Type::StringArray(create_type_info()))),
            "float" => Some(Type::Float(create_type_info())),
            name => create_object_type(name),
        }
    }
}

trait ExtractType {
    fn extract_type(&self) -> Option<String>;
}

impl ExtractType for Option<String> {
    fn extract_type(&self) -> Option<String> {
        self.as_ref().and_then(|t| {
            let re = RegexBuilder::new(r".*Array of (\w+) objects.*")
                .case_insensitive(true)
                .build()
                .unwrap();

            let cap = re.captures(t)?;

            cap.get(1).map(|m| m.as_str().to_camel())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex() {
        let input = Some("Array of result objects- see table below".to_string());
        let res = input.extract_type();
        assert_eq!(res.unwrap(), "Result");
    }
}
