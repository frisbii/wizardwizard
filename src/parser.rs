use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, char};
use nom::combinator::all_consuming;
use nom::sequence::{preceded, separated_pair};
use nom::IResult;
use std::fs;
use yaml_rust2::{yaml::Hash, Yaml, YamlLoader};

#[derive(Debug)]
pub struct Location {
    pub title: String,
    pub actions: Vec<Action>,
    pub description: String,
}

#[derive(Debug)]
pub struct Action {
    pub title: String,
    pub condition: Condition,
    pub directives: Vec<Directive>,
}

#[derive(Debug)]
pub enum Directive {
    SetProperty(PropertyId, PropertyValue),
    GoTo(LocationId),
}

#[derive(Debug)]
pub enum Condition {
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
    IsPropertyTrue(PropertyId),
}

#[derive(Debug)]
pub struct ItemId(pub u32);
#[derive(Debug)]
pub struct PropertyId(pub String);
pub type PropertyValue = bool;
#[derive(Debug)]
pub struct LocationId(pub String);

pub trait YamlExt {
    fn expect_hash<'a>(&'a self, message: &str) -> &'a Hash;

    fn expect_array<'a>(&'a self, message: &str) -> &'a Vec<Yaml>;

    fn expect_string<'a>(&'a self, message: &str) -> &'a String;
}

impl YamlExt for Yaml {
    fn expect_hash<'a>(&'a self, message: &str) -> &'a Hash {
        let Yaml::Hash(hash) = self else {
            panic!("{}", message);
        };
        hash
    }

    fn expect_string<'a>(&'a self, message: &str) -> &'a String {
        let Yaml::String(string) = self else {
            panic!("{}", message);
        };
        string
    }

    fn expect_array<'a>(&'a self, message: &str) -> &'a Vec<Yaml> {
        let Yaml::Array(array) = self else {
            panic!("{}", message);
        };
        array
    }
}

trait IResultExt<I, O, P, E> {
    fn map_val(self, f: impl Fn(O) -> P) -> IResult<I, P, E>;
}

impl<I, O, P, E> IResultExt<I, O, P, E> for IResult<I, O, E> {
    fn map_val(self, f: impl Fn(O) -> P) -> IResult<I, P, E> {
        self.map(|(remainder, parsed)| (remainder, f(parsed)))
    }
}

fn parse_is_property_true(input: &str) -> IResult<&str, Condition> {
    alphanumeric1(input)
        .map_val(|property_id| Condition::IsPropertyTrue(PropertyId(property_id.into())))
}

fn parse_not(input: &str) -> IResult<&str, Condition> {
    preceded(char('!'), parse_condition)(input).map_val(|v| Condition::Not(Box::new(v)))
}

fn parse_or(input: &str) -> IResult<&str, Condition> {
    separated_pair(parse_condition, tag(" | "), parse_condition)(input)
        .map_val(|(a, b)| Condition::Or(Box::new(a), Box::new(b)))
}

fn parse_and(input: &str) -> IResult<&str, Condition> {
    separated_pair(parse_condition, tag(" & "), parse_condition)(input)
        .map_val(|(a, b)| Condition::And(Box::new(a), Box::new(b)))
}

fn parse_condition(input: &str) -> IResult<&str, Condition> {
    alt((
        all_consuming(parse_is_property_true),
        all_consuming(parse_not),
    ))(input)
}

fn parse_directive(yaml_directive: &Yaml) -> Directive {
    let mut words = yaml_directive
        .expect_string("Directive should be a string.")
        .split_whitespace();

    match words.next() {
        Some("set") => {
            let property_id = words
                .next()
                .expect("Set directive should have a property id.");
            let property_value = words
                .next()
                .expect("Set directive should have a property value.");
            let property_boolean = match property_value {
                "true" => true,
                "false" => false,
                _ => panic!("Set directive property value should either be true or false"),
            };
            Directive::SetProperty(PropertyId(property_id.into()), property_boolean)
        }
        Some("goto") => {
            let location_id = words
                .next()
                .expect("GoTo directive should have a location id.");
            Directive::GoTo(LocationId(location_id.into()))
        }
        _ => panic!("Directive first word should either be set or goto."),
    }
}

fn parse_action(yaml_action: &Yaml) -> Action {
    println!("{:#?}", yaml_action);

    let mut title_details_mapping: Hash = yaml_action
        .expect_hash("Action should be a YAML mapping.")
        .clone();

    let (yaml_title, yaml_details) = title_details_mapping
        .pop_front()
        .expect("Action should have a title-details pair.");

    let mut condition_directive_mapping: Hash = yaml_details
        .expect_hash("Action details should be a YAML mapping.")
        .clone();

    let (yaml_condition, yaml_directives) = condition_directive_mapping
        .pop_front()
        .expect("Action should have a condition-directive pair.");

    println!("{:#?}", yaml_directives);

    let condition_string = yaml_condition.expect_string("Condition should be a string.");

    let directives: Vec<Directive> = yaml_directives
        .expect_array("Action directives should be an array.")
        .iter()
        .map(parse_directive)
        .collect();

    Action {
        title: yaml_title
            .expect_string("Action tile should be a string.")
            .into(),
        condition: parse_condition(condition_string.as_str())
            .expect("Condition should parse correctly.")
            .1,
        directives,
    }
}

fn parse_location(source: &str, title: &str) -> Location {
    let docs = YamlLoader::load_from_str(source).expect("Location file should be in YAML format.");
    let doc = &docs[0];

    let mapping = doc.expect_hash("Location should be a YAML mapping.");

    let description = mapping
        .get(&Yaml::String("description".into()))
        .expect("There should be a description.")
        .expect_string("Location description should be a string.");

    let default = Yaml::Array(vec![]);
    let yaml_actions = mapping
        .get(&Yaml::String("actions".into()))
        .unwrap_or(&default);

    let actions: Vec<Action> = yaml_actions
        .expect_array("Location actions should be an array.")
        .iter()
        .map(parse_action)
        .collect::<Vec<Action>>();

    Location {
        title: title.into(),
        actions,
        description: description.into(),
    }
}

fn load_locations() -> Vec<Location> {
    let paths = fs::read_dir("assets/").unwrap();
    paths
        .map(|path_result| {
            let path = path_result.expect("Path should exist.").path();
            parse_location(
                fs::read_to_string(&path)
                    .expect("Should be able to read from location file.")
                    .as_str(),
                path.file_stem()
                    .expect("Location file should have a name.")
                    .to_str()
                    .expect("Location file name should be valid."),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_location() {
        let s = "
description: Lorem ipsum dolor sit amet.

actions:
- unlock door:
    hasKey:
    - set isDoorOpen true
    - goto nextRoom
";
        let locations = load_locations();
        println!("{:#?}", locations);
    }
}
