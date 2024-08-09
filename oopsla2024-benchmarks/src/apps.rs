use arbitrary::Unstructured;
use cedar_policy_core::{
    ast::{Entity, EntityUID, Id, PolicyID, PolicySet},
    entities::{EntityUidJson, JsonDeserializationErrorContext},
    parser::parse_policyset,
};
use cedar_policy_generators::{schema::Schema as GeneratorSchema, settings::ABACSettings};
use cedar_policy_validator::{SchemaFragment, ValidatorSchema};
use std::{fs::File, path::Path, process::Command};

/// Everything we need for an example application used as a benchmark
pub struct ExampleApp {
    /// Name of the example app
    pub name: &'static str,
    /// Schema for the example application
    pub schema: GeneratorSchema,
    /// Static policies (and templates) for the example application
    pub static_policies: PolicySet,
    /// Filename of the OpenFGA authorization model for this application
    pub openfga_authz_model_filename: &'static str,
    /// Closure for converting a Cedar EntityUID to the corresponding OpenFGA Object
    pub convert_euid: Box<dyn Fn(&EntityUID) -> String>,
    /// Bespoke entity generator for this app
    pub bespoke_generator: BespokeGenerator,
}

/// Given an argument representing the number of entities per type to generate,
/// generate Cedar entites and perhaps Cedar template links
pub type BespokeGenerator = Box<
    dyn Fn(&cedar_policy_validator::ValidatorSchema, usize) -> (Vec<Entity>, Vec<TemplateLink>),
>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateLink {
    /// Name of the template
    pub template_name: String,
    /// Principal for the link
    pub principal: EntityUID,
    /// Resource for the link
    pub resource: EntityUID,
}

impl ExampleApp {
    /// The GitHub example application
    pub fn github(u: &mut Unstructured<'_>) -> Self {
        Self {
            name: "github",
            schema: {
                let schema_path =
                    Path::new("./openfga-examples/github").join("github.cedarschema.json");
                Self::load_schema(schema_path, u)
            },
            static_policies: {
                let policies_path = Path::new("./openfga-examples/github").join("policies.cedar");
                Self::load_policies(policies_path)
            },
            openfga_authz_model_filename:
                "./openfga-examples/openfga/github/authorization-model.json",
            convert_euid: Box::new(convert_github_euid),
            bespoke_generator: separate_process_bespoke_generator(
                "./generators/github_entity_generator.py",
                |num_entities| {
                    vec![
                        num_entities.to_string(), // num_repos
                        num_entities.to_string(), // num_users
                        num_entities.to_string(), // num_teams
                        num_entities.to_string(), // num_orgs
                        "0.05".to_string(),       // connection_probability
                    ]
                },
            ),
        }
    }

    /// The GitHub example application, alternate templates encoding for Cedar
    pub fn github_templates(u: &mut Unstructured<'_>) -> Self {
        Self {
            name: "github-templates",
            schema: {
                let schema_path = Path::new("./openfga-examples/github-templates")
                    .join("github-templates.cedarschema.json");
                Self::load_schema(schema_path, u)
            },
            static_policies: {
                let policies_path =
                    Path::new("./openfga-examples/github-templates").join("policies.cedar");
                Self::load_policies(policies_path)
            },
            openfga_authz_model_filename:
                "./openfga-examples/openfga/github/authorization-model.json",
            convert_euid: Box::new(convert_github_euid),
            bespoke_generator: separate_process_bespoke_generator(
                "./generators/github_templates_entity_generator.py",
                |num_entities| {
                    vec![
                        num_entities.to_string(), // num_repos
                        num_entities.to_string(), // num_users
                        num_entities.to_string(), // num_teams
                        num_entities.to_string(), // num_orgs
                        "0.05".to_string(),       // connection_probability
                    ]
                },
            ),
        }
    }

    /// The gdrive example application
    pub fn gdrive(u: &mut Unstructured<'_>) -> Self {
        Self {
            name: "gdrive",
            schema: {
                let schema_path =
                    Path::new("./openfga-examples/gdrive").join("gdrive.cedarschema.json");
                Self::load_schema(schema_path, u)
            },
            static_policies: {
                let policies_path = Path::new("./openfga-examples/gdrive").join("policies.cedar");
                Self::load_policies(policies_path)
            },
            openfga_authz_model_filename:
                "./openfga-examples/openfga/gdrive/authorization-model.json",
            convert_euid: Box::new(convert_gdrive_euid),
            bespoke_generator: separate_process_bespoke_generator(
                "./generators/gdrive_entity_generator.py",
                |num_entities| {
                    vec![
                        num_entities.to_string(), // num_users
                        num_entities.to_string(), // num_groups
                        num_entities.to_string(), // num_docs
                        num_entities.to_string(), // num_folders
                        "0.05".to_string(),       // connection_probability
                    ]
                },
            ),
        }
    }

    /// The gdrive example application, alternate templates encoding for Cedar
    pub fn gdrive_templates(u: &mut Unstructured<'_>) -> Self {
        Self {
            name: "gdrive-templates",
            schema: {
                let schema_path = Path::new("./openfga-examples/gdrive-templates")
                    .join("gdrive-templates.cedarschema.json");
                Self::load_schema(schema_path, u)
            },
            static_policies: {
                let policies_path =
                    Path::new("./openfga-examples/gdrive-templates").join("policies.cedar");
                Self::load_policies(policies_path)
            },
            openfga_authz_model_filename:
                "./openfga-examples/openfga/gdrive/authorization-model.json",
            convert_euid: Box::new(convert_gdrive_euid),
            bespoke_generator: separate_process_bespoke_generator(
                "./generators/gdrive_templates_entity_generator.py",
                |num_entities| {
                    vec![
                        num_entities.to_string(), // num_users
                        num_entities.to_string(), // num_groups
                        num_entities.to_string(), // num_docs
                        num_entities.to_string(), // num_folders
                        "0.05".to_string(),       // connection_probability
                    ]
                },
            ),
        }
    }

    /// The tinytodo example application
    pub fn tinytodo(u: &mut Unstructured<'_>) -> Self {
        Self {
            name: "tinytodo",
            schema: {
                let schema_path = Path::new("./tinytodo").join("tinytodo.cedarschema.json");
                Self::load_schema(schema_path, u)
            },
            static_policies: {
                let policies_path = Path::new("./tinytodo").join("tinytodo.cedar");
                parse_policyset(
                    &std::fs::read_to_string(policies_path)
                        .expect("failed to read tinytodo policies file"),
                )
                .expect("failed to parse tinytodo policies")
            },
            openfga_authz_model_filename: "tinytodo/openfga/authorization-model.json",
            convert_euid: Box::new(convert_tinytodo_euid),
            bespoke_generator: separate_process_bespoke_generator(
                "./generators/tinytodo_entity_generator.py",
                |num_entities| {
                    vec![
                        num_entities.to_string(), // num_users
                        num_entities.to_string(), // num_teams
                        num_entities.to_string(), // num_lists
                        "0.05".to_string(),       // connection_probability
                    ]
                },
            ),
        }
    }

    /// Load policies and templates from the given filepath, using `id`
    /// annotation for the policy ID if present
    pub fn load_policies(path: impl AsRef<Path>) -> PolicySet {
        use std::str::FromStr;
        let pset = parse_policyset(
            &std::fs::read_to_string(path.as_ref()).expect("failed to read policies file"),
        )
        .expect("failed to parse policies");
        // new_ps will contain the same static policies and templates,
        // but policies' and templates' ids will be renamed to match
        // their "id" annotation, if present
        let mut new_pset = PolicySet::new();
        let id_key = Id::from_str("id").unwrap();
        let templates = pset.templates().map(|t| match t.annotation(&id_key) {
            None => t.clone(),
            Some(anno) => t.new_id(PolicyID::from_smolstr(anno.clone())),
        });
        for template in templates {
            new_pset.add_template(template).unwrap();
        }
        let policies = pset.policies().map(|p| match p.annotation(&id_key) {
            None => p.clone(),
            Some(anno) => p.new_id(PolicyID::from_smolstr(anno.clone())),
        });
        for policy in policies {
            new_pset.add(policy).unwrap();
        }
        new_pset
    }

    /// Create a `GeneratorSchema` from the given filepath
    pub fn load_schema(path: impl AsRef<Path>, u: &mut Unstructured<'_>) -> GeneratorSchema {
        let schema = SchemaFragment::from_json_file(File::open(path.as_ref()).unwrap_or_else(|e| {
            panic!(
                "failed to open schema file {}: {e}",
                path.as_ref().display()
            )
        }))
        .expect("failed to parse schema");
        let settings = ABACSettings {
            match_types: true,
            enable_extensions: true,
            max_depth: 3,
            max_width: 3,
            enable_additional_attributes: true,
            enable_like: true,
            enable_action_groups_and_attrs: true,
            enable_arbitrary_func_call: false,
            enable_unknowns: false,
            enable_unspecified_apply_spec: false,
            enable_action_in_constraints: false,
        };
        GeneratorSchema::from_schemafrag(schema, settings, u).expect("failed to generate schema")
    }

    /// Get the `ValidatorSchema` for this `ExampleApp`
    pub fn validator_schema(&self) -> ValidatorSchema {
        ValidatorSchema::try_from(self.schema.clone())
            .unwrap_or_else(|e| panic!("failed to convert schema for {}: {e}", self.name))
    }
}

/// Construct a BespokeGenerator that calls the given command with the given arguments.
/// The command is expected to produce entity JSON on stdout.
fn separate_process_bespoke_generator(
    command_path: impl AsRef<Path> + 'static,
    args: impl Fn(usize) -> Vec<String> + 'static,
) -> BespokeGenerator {
    use cedar_policy_core::entities::{EntityJsonParser, TCComputation};
    use cedar_policy_core::extensions::Extensions;
    use std::io::Write;
    Box::new(move |cedar_schema, num_entities| {
        let mut command = Command::new(command_path.as_ref());
        let output = command
            .args(args(num_entities))
            .output()
            .unwrap_or_else(|e| panic!("{} failed: {e}", command_path.as_ref().display()));
        if !output.status.success() {
            if output.stderr.is_empty() {
                if output.stdout.is_empty() {
                    panic!(
                        "{} produced an error with code {}, but no text on stderr or stdout",
                        command_path.as_ref().display(),
                        output.status.code().unwrap()
                    );
                } else {
                    eprintln!("{} produced an error with code {} and no text on stderr, but this text on stdout:", command_path.as_ref().display(), output.status.code().unwrap());
                    std::io::stderr()
                        .write_all(&output.stdout)
                        .expect("failed to write to stdout");
                    panic!("see above");
                }
            } else {
                eprintln!(
                    "{} produced an error with code {}:",
                    command_path.as_ref().display(),
                    output.status.code().unwrap()
                );
                std::io::stderr()
                    .write_all(&output.stderr)
                    .expect("failed to write to stderr");
                panic!("see above");
            }
        }
        if output.stdout.is_empty() {
            panic!(
                "{} succeeded but produced no output",
                command_path.as_ref().display()
            );
        }
        let json: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|e| panic!("bad json from {}: {e}", command_path.as_ref().display()));
        // we need to split the json into entities with the special type TemplateLink
        // (which aren't entities at all, but need to be converted to template links)
        // and all other entities, which are actual entity data
        let (links, entities) = json.into_iter().partition(|m| {
            let pretty_panic = |s: &str| panic!("{s}:\n{m}");
            m.get("uid")
                .unwrap_or_else(|| pretty_panic("entity should have uid key"))
                .get("__entity")
                .unwrap_or_else(|| pretty_panic("uid should have __entity key"))
                .get("type")
                .unwrap_or_else(|| pretty_panic("__entity key should have a 'type' subkey"))
                == "TemplateLink"
        });
        let coreschema = cedar_policy_validator::CoreSchema::new(cedar_schema);
        let eparser = EntityJsonParser::new(
            Some(&coreschema),
            Extensions::all_available(),
            TCComputation::AssumeAlreadyComputed, // we explicitly don't want TC computed (yet) -- eg TC shouldn't be computed before sending to OpenFGA
        );
        let entities = eparser
            .from_json_value(serde_json::Value::Array(entities))
            .unwrap_or_else(|e| {
                panic!(
                    "bad json from {}: {e}\njson was {}",
                    command_path.as_ref().display(),
                    String::from_utf8(output.stdout).unwrap(),
                )
            });
        let parse_euid_json = |val: serde_json::Value| -> EntityUID {
            let euidjson: EntityUidJson = serde_json::from_value(val).expect("uid should be valid");
            euidjson
                .into_euid(|| JsonDeserializationErrorContext::EntityUid)
                .expect("uid should be valid")
        };
        let links = links
            .into_iter()
            .map(|link| {
                let link_attrs = link.get("attrs").expect("link should have attrs");
                let template_name = link_attrs
                    .get("template_name")
                    .expect("link should have template_name")
                    .as_str()
                    .expect("template_name should be string")
                    .to_string();
                let principal = parse_euid_json(
                    link_attrs
                        .get("principal")
                        .expect("link should have principal")
                        .clone(),
                );
                let resource = parse_euid_json(
                    link_attrs
                        .get("resource")
                        .expect("link should have resource")
                        .clone(),
                );
                TemplateLink {
                    template_name,
                    principal,
                    resource,
                }
            })
            .collect();
        (entities.into_iter().collect(), links)
    })
}

/// Convert a Cedar EntityUID to the corresponding OpenFGA Object, in the
/// context of the github example
fn convert_github_euid(euid: &EntityUID) -> String {
    euid.to_string()
        .replace("Repository::", "repo:")
        .replace("User::", "user:")
        .replace("Team::", "team:")
        .replace("Organization::", "organization:")
        .replace(r#"Action::"read""#, "reader")
        .replace(r#"Action::"triage""#, "triager")
        .replace(r#"Action::"write""#, "writer")
        .replace(r#"Action::"maintain""#, "maintainer")
        .replace(r#"Action::"admin""#, "admin")
}

/// Convert a Cedar EntityUID to the corresponding OpenFGA Object, in the
/// context of the gdrive example
fn convert_gdrive_euid(euid: &EntityUID) -> String {
    euid.to_string()
        .replace("User::", "user:")
        .replace("Group::", "group:")
        .replace("Document::", "doc:")
        .replace("Folder::", "folder:")
        .replace(r#"Action::"createDocument""#, "can_create_file")
        .replace(r#"Action::"changeOwner""#, "can_change_owner")
        .replace(r#"Action::"share""#, "can_share")
        .replace(r#"Action::"write""#, "can_write")
        .replace(r#"Action::"read""#, "can_read")
}

/// Convert a Cedar EntityUID to the corresponding OpenFGA Object, in the
/// context of the tinytodo example
fn convert_tinytodo_euid(euid: &EntityUID) -> String {
    euid.to_string()
        .replace("User::", "user:")
        .replace("Team::", "team:")
        .replace("List::", "list:")
        .replace("Application::", "application:")
        .replace(r#"Action::"CreateList""#, "create_list")
        .replace(r#"Action::"GetList""#, "get_list")
        .replace(r#"Action::"UpdateList""#, "update_list")
        .replace(r#"Action::"DeleteList""#, "delete_list")
        .replace(r#"Action::"GetLists""#, "get_lists")
        .replace(r#"Action::"CreateTask""#, "create_task")
        .replace(r#"Action::"UpdateTask""#, "update_task")
        .replace(r#"Action::"DeleteTask""#, "delete_task")
        .replace(r#"Action::"EditShares""#, "edit_shares")
}
