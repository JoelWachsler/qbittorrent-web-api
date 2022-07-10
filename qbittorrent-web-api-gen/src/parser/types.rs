use std::collections::HashMap;

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
    pub is_list: bool,
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

pub const OPTIONAL: &str = "_optional_";

#[derive(Debug, Clone)]
pub enum Type {
    Number(TypeInfo),
    Float(TypeInfo),
    Bool(TypeInfo),
    String(TypeInfo),
    StringArray(TypeInfo),
    Object(TypeInfo),
}

impl Type {
    pub fn to_owned_type(&self) -> String {
        match self {
            Type::Number(_) => "i128".into(),
            Type::Float(_) => "f32".into(),
            Type::Bool(_) => "bool".into(),
            Type::String(_) => "String".into(),
            // TODO: fixme
            Type::StringArray(_) => "String".into(),
            Type::Object(_) => "String".into(),
        }
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

    pub fn get_type_info(&self) -> &TypeInfo {
        match self {
            Type::Number(t) => t,
            Type::Float(t) => t,
            Type::Bool(t) => t,
            Type::String(t) => t,
            Type::StringArray(t) => t,
            Type::Object(t) => t,
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
        let type_info = TypeInfo::new(type_name, is_optional, false, description, available_types);

        match type_as_str {
            "bool" => Some(Type::Bool(type_info)),
            "integer" | "number" | "int" => Some(Type::Number(type_info)),
            "string" => Some(Type::String(type_info)),
            // This is probably not right but we don't have any information about the actual type.
            "array" => Some(Type::StringArray(type_info)),
            "object" => Some(Type::Object(type_info)),
            "float" => Some(Type::Float(type_info)),
            _ => None,
        }
    }
}
