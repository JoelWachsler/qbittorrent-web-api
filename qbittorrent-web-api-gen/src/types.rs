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
    pub description: Option<String>,
    is_optional: bool,
    is_list: bool,
}

impl TypeInfo {
    pub fn new(name: &str, is_optional: bool, is_list: bool, description: Option<String>) -> Self {
        Self {
            name: name.into(),
            description,
            is_optional,
            is_list,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefType {
    String(String),
    Map(String, String),
}

#[derive(Debug, Clone)]
pub struct Object {
    pub type_info: TypeInfo,
    pub ref_type: RefType,
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
    pub fn to_borrowed_type(&self) -> String {
        match self {
            Type::Number(_) => "i32".into(),
            Type::Float(_) => "f32".into(),
            Type::Bool(_) => "bool".into(),
            Type::String(_) => "str".into(),
            Type::StringArray(_) => "&[str]".into(),
            Type::Object(_) => todo!(),
        }
    }

    pub fn should_borrow(&self) -> bool {
        matches!(self, Type::String(_) | Type::Object(_))
    }

    pub fn is_optional(&self) -> bool {
        self.get_type_info().is_optional
    }

    pub fn is_list(&self) -> bool {
        self.get_type_info().is_list
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

    pub fn from(type_as_str: &str, name: &str, description: Option<String>) -> Option<Type> {
        let type_name = match name.split_once(OPTIONAL) {
            Some((split, _)) => split,
            None => name,
        }
        .trim();

        let is_optional = name.contains(OPTIONAL);
        let is_list = description
            .clone()
            .map(|desc| desc.contains("array"))
            .unwrap_or(false);

        let (type_without_array, type_contains_array) = if type_as_str.contains("array") {
            (type_as_str.replace("array", ""), true)
        } else {
            (type_as_str.to_owned(), false)
        };

        let create_type_info = || {
            TypeInfo::new(
                type_name,
                is_optional,
                is_list || type_contains_array,
                description.clone(),
            )
        };

        let create_object_type = |ref_type: RefType| {
            Some(Type::Object(Object {
                type_info: create_type_info(),
                ref_type,
            }))
        };

        match type_without_array.trim() {
            "raw" => None,
            "bool" => Some(Type::Bool(create_type_info())),
            "integer" | "number" | "int" => Some(Type::Number(create_type_info())),
            "string" => Some(Type::String(create_type_info())),
            "array" => description
                .extract_type()
                .and_then(create_object_type)
                .or_else(|| Some(Type::StringArray(create_type_info()))),
            "float" => Some(Type::Float(create_type_info())),
            name => description
                .extract_type()
                .and_then(create_object_type)
                .or_else(|| {
                    let n = if name.is_empty() {
                        "String".into()
                    } else {
                        name.into()
                    };

                    create_object_type(RefType::String(n))
                }),
        }
    }
}

trait ExtractType {
    fn extract_type(&self) -> Option<RefType>;
}

impl ExtractType for Option<String> {
    fn extract_type(&self) -> Option<RefType> {
        let list_type = || {
            self.as_ref().and_then(|t| {
                let re = RegexBuilder::new(r"(?:Array|List) of (\w+) objects")
                    .case_insensitive(true)
                    .build()
                    .unwrap();

                let cap = re.captures(t)?;

                cap.get(1)
                    .map(|m| m.as_str().to_camel())
                    .map(RefType::String)
            })
        };

        let map_type = || {
            self.as_ref().and_then(|t| {
                let re = RegexBuilder::new(r"map from (\w+) to (\w+) object")
                    .case_insensitive(true)
                    .build()
                    .unwrap();

                let cap = re.captures(t)?;
                let key_type = match cap.get(1).map(|m| m.as_str().to_camel()) {
                    Some(k) => k,
                    None => return None,
                };
                let value_type = match cap.get(2).map(|m| m.as_str().to_camel()) {
                    Some(v) => v,
                    None => return None,
                };

                Some(RefType::Map(key_type, value_type))
            })
        };

        let object_type = || {
            self.as_ref().and_then(|t| {
                let re = RegexBuilder::new(r"(\w+) object see table below")
                    .case_insensitive(true)
                    .build()
                    .unwrap();

                let cap = re.captures(t)?;
                let object_type = match cap.get(1).map(|m| m.as_str().to_camel()) {
                    Some(k) => k,
                    None => return None,
                };

                Some(RefType::String(object_type))
            })
        };

        list_type().or_else(map_type).or_else(object_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex() {
        let input = Some("Array of result objects- see table below".to_string());
        let res = input.extract_type();
        assert_eq!(res.unwrap(), RefType::String("Result".into()));
    }
}
