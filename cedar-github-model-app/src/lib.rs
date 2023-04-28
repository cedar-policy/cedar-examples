#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

use anyhow::{Context as _, Error, Result};
use cedar_policy::*;
use clap::{AppSettings, Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::{
    fs::OpenOptions,
    path::Path,
    process::{ExitCode, Termination},
    str::FromStr,
    time::Instant,
};

/// Basic Cedar CLI for evaluating authorization queries
#[derive(Debug, Parser)]
#[clap(version, about, long_about = None)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Evaluate an authorization query
    Benchmark(BenchmarkArgs),
}

#[derive(Args, Debug)]
pub struct BenchmarkArgs {
    /// File containing the Cedar policies to evaluate against
    #[clap(long = "policies", value_name = "FILE")]
    pub policies_file: String,
    /// File containing template instances
    #[clap(long = "instances", value_name = "FILE")]
    pub instances_file: Option<String>,
    /// File containing schema information
    // Used for schema-based parsing of entity hierarchy, if present
    #[clap(long = "schema", value_name = "FILE")]
    pub schema_file: Option<String>,
    /// File containing JSON representation of the Cedar entity hierarchy
    /// If not present, a random entity hierarchy will be generated
    #[clap(long = "entities", value_name = "FILE")]
    pub entities_file: Option<String>,
    #[clap(short, long)]
    pub verbose: bool,
    /// Time authorization and report timing information
    #[clap(short, long)]
    pub timing: bool,
    /// Perform entity slicing
    #[clap(short, long)]
    pub slicing: bool,
    /// Number of queries to generate
    #[clap(short, long)]
    pub num_queries: u32,
    /// Whether to print the queries that get "allow"
    #[clap(short, long)]
    pub print_allows: bool,
}

fn read_policy_and_instances(
    policies_filename: impl AsRef<Path>,
    instances_filename: Option<impl AsRef<Path>>,
) -> Result<PolicySet> {
    let mut pset = read_policy_file(policies_filename.as_ref())?;
    if let Some(instances_filename) = instances_filename {
        add_instances_to_set(instances_filename.as_ref(), &mut pset)?;
    }
    Ok(pset)
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "LiteralInstance")]
#[serde(into = "LiteralInstance")]
struct Instance {
    template_id: String,
    instance_id: String,
    args: HashMap<SlotId, String>,
}

impl TryFrom<LiteralInstance> for Instance {
    type Error = String;

    fn try_from(value: LiteralInstance) -> Result<Self, Self::Error> {
        Ok(Self {
            template_id: value.template_id,
            instance_id: value.instance_id,
            args: value
                .args
                .into_iter()
                .map(|(k, v)| parse_slot_id(k).map(|slot_id| (slot_id, v)))
                .collect::<Result<HashMap<SlotId, String>, Self::Error>>()?,
        })
    }
}

fn parse_slot_id<S: AsRef<str>>(s: S) -> Result<SlotId, String> {
    match s.as_ref() {
        "?principal" => Ok(SlotId::principal()),
        "?resource" => Ok(SlotId::principal()),
        _ => Err(format!(
            "Invalid SlotId! Expected ?principal|?resource, got: {}",
            s.as_ref()
        )),
    }
}

#[derive(Serialize, Deserialize)]
struct LiteralInstance {
    template_id: String,
    instance_id: String,
    args: HashMap<String, String>,
}

impl From<Instance> for LiteralInstance {
    fn from(i: Instance) -> Self {
        Self {
            template_id: i.template_id,
            instance_id: i.instance_id,
            args: i
                .args
                .into_iter()
                .map(|(k, v)| (format!("{k}"), v))
                .collect(),
        }
    }
}

/// Iterate over instances in the instance file and add them to the set
fn add_instances_to_set(path: impl AsRef<Path>, policy_set: &mut PolicySet) -> Result<()> {
    for instance in load_instance_file(path)? {
        let slotenv = create_slot_env(&instance.args)?;
        policy_set.link(
            PolicyId::from_str(&instance.template_id)?,
            PolicyId::from_str(&instance.instance_id)?,
            slotenv,
        )?;
    }
    Ok(())
}

fn create_slot_env(data: &HashMap<SlotId, String>) -> Result<HashMap<SlotId, EntityUid>> {
    data.iter()
        .map(|(key, value)| Ok(EntityUid::from_str(value).map(|euid| (key.clone(), euid))?))
        .collect::<Result<HashMap<SlotId, EntityUid>>>()
}

/// Read instance set to a Vec
fn load_instance_file(path: impl AsRef<Path>) -> Result<Vec<Instance>> {
    let f = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => {
            // If the file doesn't exist, then give back the empty entity set
            return Ok(vec![]);
        }
    };
    if f.metadata().context("Failed to read metadata")?.len() == 0 {
        // File is empty, return empty set
        Ok(vec![])
    } else {
        // File has contents, deserialize
        serde_json::from_reader(f).context("Deser error")
    }
}

/// Add a single instance to the instance file
fn update_instance_file(path: impl AsRef<Path>, new_instance: Instance) -> Result<()> {
    let mut instances = load_instance_file(path.as_ref())?;
    instances.push(new_instance);
    write_instance_file(&instances, path.as_ref())
}

/// Write a slice of instances to the instance file
fn write_instance_file(instances: &[Instance], path: impl AsRef<Path>) -> Result<()> {
    let f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;
    Ok(serde_json::to_writer(f, instances)?)
}

fn read_policy_file(filename: impl AsRef<Path>) -> Result<PolicySet> {
    let src = std::fs::read_to_string(filename.as_ref()).context(format!(
        "failed to open policy file {}",
        filename.as_ref().display()
    ))?;
    PolicySet::from_str(&src).context(format!(
        "failed to parse policies from file {}",
        filename.as_ref().display()
    ))
}

fn read_schema_file(filename: impl AsRef<Path>) -> Result<Schema> {
    let schema_src = std::fs::read_to_string(filename.as_ref()).context(format!(
        "failed to open schema file {}",
        filename.as_ref().display()
    ))?;
    Schema::from_str(&schema_src).context(format!(
        "failed to parse schema from file {}",
        filename.as_ref().display()
    ))
}

#[derive(Eq, PartialEq, Debug)]
pub enum CedarExitCode {
    // The command completed successfully with a result other than a
    // authorization deny or validation failure.
    Success,
    // The command failed to complete successfully.
    Failure,
    // The command completed successfully, but the result of the authorization
    // query was DENY.
    AuthorizeDeny,
    // The command completed successfully, but it detected a validation failure
    // in the given schema and policies.
    ValidationFailure,
}

impl Termination for CedarExitCode {
    fn report(self) -> ExitCode {
        match self {
            CedarExitCode::Success => ExitCode::SUCCESS,
            CedarExitCode::Failure => ExitCode::FAILURE,
            CedarExitCode::AuthorizeDeny => ExitCode::from(2),
            CedarExitCode::ValidationFailure => ExitCode::from(3),
        }
    }
}

pub fn benchmark(args: &BenchmarkArgs) -> CedarExitCode {
    println!();
    let ans = benchmark_inner(
        &args.policies_file,
        args.instances_file.as_ref(),
        args.schema_file.as_ref(),
        args.entities_file.as_ref(),
        args.timing,
        args.slicing,
        args.num_queries,
        args.print_allows,
    );
    match ans {
        Ok(ans) => {
            let status = match ans.decision() {
                Decision::Allow => {
                    println!("ALLOW");
                    CedarExitCode::Success
                }
                Decision::Deny => {
                    println!("DENY");
                    CedarExitCode::AuthorizeDeny
                }
            };
            if ans.diagnostics().errors().peekable().peek().is_some() {
                println!();
                for err in ans.diagnostics().errors() {
                    println!("{}", err);
                }
            }
            if args.verbose {
                println!();
                if ans.diagnostics().reason().peekable().peek().is_none() {
                    println!("note: no policies applied to this query");
                } else {
                    println!("note: this decision was due to the following policies:");
                    for reason in ans.diagnostics().reason() {
                        println!("  {}", reason);
                    }
                    println!();
                }
            }
            status
        }
        Err(errs) => {
            for err in errs {
                println!("{:#}", err);
            }
            CedarExitCode::Failure
        }
    }
}

/// Load an `Entities` object from the given JSON filename and optional schema.
fn load_entities(entities_filename: impl AsRef<Path>, schema: Option<&Schema>) -> Result<Entities> {
    match std::fs::OpenOptions::new()
        .read(true)
        .open(entities_filename.as_ref())
    {
        Ok(f) => Entities::from_json_file(f, schema).context(format!(
            "failed to parse entities from file {}",
            entities_filename.as_ref().display()
        )),
        Err(e) => Err(e).context(format!(
            "failed to open entities file {}",
            entities_filename.as_ref().display()
        )),
    }
}

fn build_entities(file_name: Option<impl AsRef<Path>>) -> Entities {
    match file_name {
        Some(f) => load_entities(f, None).unwrap(),
        None => Entities::empty(),
    }
}

fn build_queries(entities: &Entities, num_queries: u32) -> Vec<Request> {
    //Assumes we have user_0...user_max_user and repo_0...repo_max_repo
    let mut max_user = 0;
    loop {
        let euid = EntityUid::from_type_name_and_id(
            EntityTypeName::from_str("User").unwrap(),
            EntityId::from_str(format!("user_{}", max_user).as_str()).unwrap(),
        );
        if entities.get(&euid).is_none() {
            break;
        }
        max_user += 1;
    }

    let mut max_repo = 0;
    loop {
        let euid = EntityUid::from_type_name_and_id(
            EntityTypeName::from_str("Repository").unwrap(),
            EntityId::from_str(format!("repo_{}", max_repo).as_str()).unwrap(),
        );
        if entities.get(&euid).is_none() {
            break;
        }
        max_repo += 1;
    }

    println!("Max num users: {}", max_user);
    println!("Max num repos: {}", max_repo);

    let mut v = Vec::new();
    let step = max_user * max_repo / num_queries;
    let mut ordinal = 0;
    for _ in 0..num_queries {
        let u = ordinal / max_repo;
        let r = ordinal % max_repo;
        let q = Request::new(
            Some(EntityUid::from_type_name_and_id(
                EntityTypeName::from_str("User").unwrap(),
                EntityId::from_str(format!("user_{}", u).as_str()).unwrap(),
            )),
            Some(EntityUid::from_type_name_and_id(
                EntityTypeName::from_str("Action").unwrap(),
                EntityId::from_str("push").unwrap(),
            )),
            Some(EntityUid::from_type_name_and_id(
                EntityTypeName::from_str("Repository").unwrap(),
                EntityId::from_str(format!("repo_{}", r).as_str()).unwrap(),
            )),
            Context::empty(),
        );
        v.push(q);
        ordinal += step;
    }
    v
}

struct EntitySlice<'a> {
    entity_slice: Vec<&'a Entity>,
    entity_uids_in_slice: BTreeSet<EntityUid>,
}

impl<'a> EntitySlice<'a> {
    pub fn new() -> EntitySlice<'a> {
        EntitySlice {
            entity_slice: Vec::new(),
            entity_uids_in_slice: BTreeSet::new(),
        }
    }

    pub fn add_entity_and_ancestors(
        &mut self,
        all_entities: &'a Entities,
        euid: EntityUid,
        query: &Request,
        dummy_entities: &mut Vec<Entity>,
    ) {
        if !self.entity_uids_in_slice.contains(&euid) {
            self.entity_slice.push(all_entities.get(&euid).unwrap());
            self.entity_uids_in_slice.insert(euid.clone());
        }

        for ancestor in all_entities.ancestors(&euid).unwrap() {
            if !self.entity_uids_in_slice.contains(ancestor) {
                //Only add repositories / repository groups if they are/belong to the resource in the query
                if ancestor.type_name().to_string().eq("Repository") {
                    if ancestor
                        .id()
                        .to_string()
                        .eq(query.resource().unwrap().id().as_ref())
                    {
                        self.entity_slice.push(all_entities.get(ancestor).unwrap());
                        self.entity_uids_in_slice.insert(ancestor.clone());
                    } else {
                        //minimal repo
                        let min_repo_euid = EntityUid::from_type_name_and_id(
                            EntityTypeName::from_str("Repository").unwrap(),
                            EntityId::from_str(ancestor.id().as_ref()).unwrap(),
                        );
                        let min_repo =
                            Entity::new(min_repo_euid.clone(), HashMap::new(), HashSet::new());
                        self.entity_uids_in_slice.insert(min_repo_euid);
                        dummy_entities.push(min_repo);
                    }
                } else if ancestor.type_name().to_string().eq("UserGroup") {
                    //"repo_x_readers, repo_x_writers etc."
                    if ancestor
                        .id()
                        .to_string()
                        .contains(query.resource().unwrap().id().as_ref())
                    {
                        self.entity_slice.push(all_entities.get(ancestor).unwrap());
                        self.entity_uids_in_slice.insert(ancestor.clone());
                    } else {
                        //minimal UserGroup
                        let min_user_group_euid = EntityUid::from_type_name_and_id(
                            EntityTypeName::from_str("UserGroup").unwrap(),
                            EntityId::from_str(ancestor.id().as_ref()).unwrap(),
                        );
                        let min_user_group = Entity::new(
                            min_user_group_euid.clone(),
                            HashMap::new(),
                            HashSet::new(),
                        );

                        self.entity_uids_in_slice.insert(min_user_group_euid);
                        dummy_entities.push(min_user_group);
                    }
                } else {
                    self.entity_slice.push(all_entities.get(ancestor).unwrap());
                    self.entity_uids_in_slice.insert(ancestor.clone());
                }
            }
        }
    }
}

fn get_entity_slice_for_query(entities: &Entities, query: &Request) -> Entities {
    //For the permissions we're modeling, we need:
    //
    //If resource is a repo
    //The user
    //The repository
    //The groups for the repository (repo_x_readers, writers, ...)
    //The org the repo is under
    //the groups for that org
    //Any transitive teams the user is in
    //
    //If the resource is an issue:
    //The user
    //The issue
    //The issue's repo
    //The groups for the repository (repo_x_readers, writers, ...)
    //The org the repo is under
    //the groups for that org
    //any transitive teams the user is in

    //Because we compute the transive closure (which could be turned off but probably shouldn't be)
    //we also need to include all ancestors of any entity listed above

    let issue_actions = vec!["assign_issue", "edit_issue", "delete_issue"];

    let mut entity_slice = EntitySlice::new();

    let action_str = query.action().unwrap().to_string();

    //We'll store dummy entities here so their lifetime lasts until we call Entities::new()
    let mut dummy_entities: Vec<Entity> = Vec::new();

    if issue_actions.contains(&action_str.as_str()) {
        println!(
            "should be an empty vec: {}",
            entity_slice.entity_slice.len()
        );
    } else {
        entity_slice.add_entity_and_ancestors(
            entities,
            query.principal().unwrap().clone(),
            query,
            &mut dummy_entities,
        );

        entity_slice.add_entity_and_ancestors(
            entities,
            query.resource().unwrap().clone(),
            query,
            &mut dummy_entities,
        );

        let repo_name = query.resource().unwrap().id().clone();
        // All of these are ancestors of _admins, so we don't need to add them directly
        entity_slice.add_entity_and_ancestors(
            entities,
            EntityUid::from_type_name_and_id(
                EntityTypeName::from_str("UserGroup").unwrap(),
                EntityId::from_str(format!("{}_readers", repo_name).as_str()).unwrap(),
            ),
            query,
            &mut dummy_entities,
        );
        entity_slice.add_entity_and_ancestors(
            entities,
            EntityUid::from_type_name_and_id(
                EntityTypeName::from_str("UserGroup").unwrap(),
                EntityId::from_str(format!("{}_triagers", repo_name).as_str()).unwrap(),
            ),
            query,
            &mut dummy_entities,
        );
        entity_slice.add_entity_and_ancestors(
            entities,
            EntityUid::from_type_name_and_id(
                EntityTypeName::from_str("UserGroup").unwrap(),
                EntityId::from_str(format!("{}_writers", repo_name).as_str()).unwrap(),
            ),
            query,
            &mut dummy_entities,
        );
        entity_slice.add_entity_and_ancestors(
            entities,
            EntityUid::from_type_name_and_id(
                EntityTypeName::from_str("UserGroup").unwrap(),
                EntityId::from_str(format!("{}_maintainers", repo_name).as_str()).unwrap(),
            ),
            query,
            &mut dummy_entities,
        );
        entity_slice.add_entity_and_ancestors(
            entities,
            EntityUid::from_type_name_and_id(
                EntityTypeName::from_str("UserGroup").unwrap(),
                EntityId::from_str(format!("{}_admins", repo_name).as_str()).unwrap(),
            ),
            query,
            &mut dummy_entities,
        );
    }

    println!("Num entities in slice: {}", entity_slice.entity_slice.len());
    println!("Num dummy entities: {}", dummy_entities.len());

    let mut slice_entities = entity_slice
        .entity_slice
        .into_iter()
        .cloned()
        .collect::<Vec<Entity>>();

    slice_entities.append(&mut dummy_entities);

    Entities::from_entities(slice_entities).unwrap()
}

/// This uses the Cedar API to call the authorization engine.
fn benchmark_inner(
    policies_filename: impl AsRef<Path>,
    instances_filename: Option<impl AsRef<Path>>,
    schema_filename: Option<impl AsRef<Path>>,
    entities_file: Option<impl AsRef<Path>>,
    compute_duration: bool,
    entity_slicing: bool,
    num_queries: u32,
    print_allows: bool,
) -> Result<Answer, Vec<Error>> {
    let mut errs = vec![];
    let policies = match read_policy_and_instances(policies_filename.as_ref(), instances_filename) {
        Ok(pset) => pset,
        Err(e) => {
            errs.push(e);
            PolicySet::new()
        }
    };
    let _schema = match schema_filename.map(read_schema_file) {
        None => None,
        Some(Ok(schema)) => Some(schema),
        Some(Err(e)) => {
            errs.push(e);
            None
        }
    };
    if errs.is_empty() {
        let entities = build_entities(entities_file);
        let queries = build_queries(&entities, num_queries);
        let authorizer = Authorizer::new();

        for q in &queries {
            let auth_start = Instant::now();
            if entity_slicing {
                let slice_start = Instant::now();
                let entity_slice = get_entity_slice_for_query(&entities, q);
                let slice_dur = slice_start.elapsed();
                if compute_duration {
                    println!("Slicing time (micro seconds) : {}", slice_dur.as_micros());
                }
                let ans = authorizer.is_authorized(q, &policies, &entity_slice);
                if print_allows && ans.decision() == Decision::Allow {
                    println!("ALLOW");
                    println!("Request: {:?}", q);
                }
            } else {
                let ans = authorizer.is_authorized(q, &policies, &entities);
                if print_allows && ans.decision() == Decision::Allow {
                    println!("ALLOW");
                    println!("Request: {:?}", q);
                }
            }
            let auth_dur = auth_start.elapsed();
            if compute_duration {
                println!(
                    "Authorization Time (including slicing if requested) (micro seconds) : {}",
                    auth_dur.as_micros()
                );
            }
        }
        let auth_start = Instant::now();
        let ans = authorizer.is_authorized(queries.last().unwrap(), &policies, &entities);
        let auth_dur = auth_start.elapsed();
        if compute_duration {
            println!(
                "Authorization Time (micro seconds) : {}",
                auth_dur.as_micros()
            );
        }
        Ok(ans)
    } else {
        Err(errs)
    }
}
