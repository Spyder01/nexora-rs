use crate::models::file_layout::MAX_PROPERTIES_COUNT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    Int8 = 0,
    Int16,
    Int32,
    Int64,
    Float8,
    Float16,
    Float32,
    Float64,
    String32,
    String64,
    String512,
    Page,
    Bool,
    InvalidType,
}


#[derive(Debug)]
pub struct PropertyBuilder {
    name: String,
    r#type: PropertyType,
    optional: bool,
}

impl PropertyBuilder {
    pub fn new(name: String, r#type: PropertyType, optional: bool) -> Self {
        Self {
            name,
            r#type,
            optional,
        }
    }
}

#[derive(Debug)]
pub struct NodeSchemaBuilder {
    pub id: u64,
    pub properties: Vec<PropertyBuilder>,
}

impl NodeSchemaBuilder {
    pub fn new(id: u64) -> Self {
        Self {
            id: id, 
            properties: Vec::new(),
        }
    }

    pub fn property(mut self, prop: PropertyBuilder) -> Self {
        if self.properties.len() >= MAX_PROPERTIES_COUNT {
            panic!("Max properties reached for node-schema.");
        }
        self.properties.push(prop);
        self
    }
}


#[derive(Debug)]
pub struct EdgeSchemaBuilder {
    pub id: u64,
    pub properties: Vec<PropertyBuilder>,
}

impl EdgeSchemaBuilder {
    pub fn new(id: u64) -> Self {
        Self {
            id: id,
            properties: Vec::new(),
        }
    }

    pub fn property(mut self, prop: PropertyBuilder) -> Self {
        if self.properties.len() >= MAX_PROPERTIES_COUNT {
            panic!("Max properties reached for node-schema.");
        }
        self.properties.push(prop);
        self
    }
}

