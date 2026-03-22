use crate::{expressions::PropertyDefinition, tokens::Token};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ComponentMetadata {
    pub token: Token,
    pub id: u8,
    pub properties: Vec<PropertyDefinition>,
}

#[derive(Default)]
pub struct Environment {
    pub components: HashMap<String, ComponentMetadata>,
}

impl Environment {
    fn get_property_definition(
        &self,
        component_name: String,
        property_name: String,
    ) -> &PropertyDefinition {
        // let component = self.variables.get(&identifier).unwrap();
        // let component_metadata = self
        //     .components
        //     .iter()
        //     .find(|(token, data)| token.lexeme == component_name)
        //     .unwrap();
        let component_metadata = self.components.get(&component_name).unwrap();
        let properties = &component_metadata.properties;

        properties
            .iter()
            .find(|prop| prop.name.lexeme == property_name)
            .unwrap()
    }
}
