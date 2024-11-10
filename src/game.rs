use crate::parser::Condition;
use crate::parser::Directive;
use crate::parser::PropertyId;

use super::parser::LocationId;
use std::collections::HashMap;

pub struct Game {
    pub location: LocationId,
    pub properties: HashMap<String, bool>,
}

impl Game {
    fn evaluate(&self, cond: Condition) -> bool {
        match cond {
            Condition::IsPropertyTrue(PropertyId(property_id)) => {
                *(self.properties.get(&property_id).unwrap_or(&false))
            }
            Condition::Not(b) => self.evaluate(*b),
            Condition::Or(b1, b2) => self.evaluate(*b1) || self.evaluate(*b2),
            Condition::And(b1, b2) => self.evaluate(*b1) && self.evaluate(*b2),
        }
    }

    fn update(&mut self, directive: Directive) {
        match directive {
            Directive::GoTo(new_location) => self.location = new_location,
            Directive::SetProperty(PropertyId(property_id), value) => {
                let prop = self.properties.entry(property_id).or_insert(false);
                *prop = value
            }
        }
    }
}
