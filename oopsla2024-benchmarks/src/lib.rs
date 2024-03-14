use arbitrary::Unstructured;
use average::{Estimate, Mean, Quantile};
use cedar_policy_core::{
    ast::{EntityType, Request},
    authorizer::{AuthorizationError, Decision},
    entities::Entities,
};
use entity_graph::OpenEntities;
use itertools::Itertools;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::{collections::BTreeMap, time::Duration};

mod apps;
mod entity_graph;
pub use crate::apps::{ExampleApp, TemplateLink};
mod utils;

pub mod tinytodo_generator;

mod cedar_engine;
pub use cedar_engine::CedarEngine;
pub use cedar_engine::CedarOptEngine;
mod openfga_engine;
pub use openfga_engine::OpenFgaEngine;
mod rego_engine;
mod rego_requests;
pub use rego_engine::RegoEngine;
mod slicing;

pub enum Engine<'a> {
    Cedar(CedarEngine),
    CedarOpt(CedarOptEngine),
    OpenFga(OpenFgaEngine<'a>),
    Rego(RegoEngine<'a, OpenEntities>),
    RegoTC(RegoEngine<'a, Entities>),
}

impl<'a> Engine<'a> {
    /// Execute the given `Request`s, producing the corresponding `SingleExecutionReport`s
    pub fn execute<'b>(
        &'b self,
        requests: Vec<Request>,
    ) -> Box<dyn Iterator<Item = SingleExecutionReport> + 'b> {
        match self {
            Self::Cedar(e) => Box::new(requests.into_iter().map(|req| e.execute(req))),
            Self::OpenFga(e) => Box::new(requests.into_iter().map(|req| e.execute(req))),
            Self::Rego(e) => Box::new(e.execute(requests)),
            Self::RegoTC(e) => Box::new(e.execute(requests)),
            Self::CedarOpt(e) => {
                let slicer = e.get_slicer();
                Box::new(
                    requests
                        .into_iter()
                        .map(move |request| e.execute(request, &slicer)),
                )
            }
        }
    }
}

/// Stats on a single run of `is_authorized()`
pub struct SingleExecutionReport {
    /// Execution time of `is_authorized()`
    pub dur: Duration,
    /// Authz decision
    pub decision: Decision,
    /// Errors reported
    pub errors: Vec<AuthorizationError>,
    /// Number of attributes of `Context`
    pub context_attrs: usize,
}

/// Stats on many runs of `is_authorized()` with different requests but same policies, hierarchy, etc
pub struct MultiExecutionReport {
    /// Mean execution time of `is_authorized()`, in microseconds
    mean_dur_micros: Mean,
    /// Median execution time of `is_authorized()`, in microseconds
    median_dur_micros: Quantile,
    /// P90 execution time of `is_authorized()`, in microseconds
    p90_dur_micros: Quantile,
    /// P99 execution time of `is_authorized()`, in microseconds
    p99_dur_micros: Quantile,
    /// Mean number of allows (per request -- this will be between 0 and 1)
    allows: Mean,
    /// Mean number of denies (per request -- this will be between 0 and 1)
    denies: Mean,
    /// Mean number of errors (per request)
    mean_num_errors: Mean,
    /// First error encountered, if any errors
    err: Option<AuthorizationError>,
    /// Mean number of attributes of `context`
    mean_context_attrs: Mean,
}

impl MultiExecutionReport {
    /// A new report with no data
    pub fn new() -> Self {
        Self {
            mean_dur_micros: Mean::new(),
            median_dur_micros: Quantile::new(0.50),
            p90_dur_micros: Quantile::new(0.90),
            p99_dur_micros: Quantile::new(0.99),
            allows: Mean::new(),
            denies: Mean::new(),
            mean_num_errors: Mean::new(),
            err: None,
            mean_context_attrs: Mean::new(),
        }
    }

    /// Add a data point to the report
    pub fn add(&mut self, single_report: SingleExecutionReport) {
        self.mean_dur_micros
            .add(f64::try_from(u32::try_from(single_report.dur.as_micros()).unwrap()).unwrap());
        self.median_dur_micros
            .add(f64::try_from(u32::try_from(single_report.dur.as_micros()).unwrap()).unwrap());
        self.p90_dur_micros
            .add(f64::try_from(u32::try_from(single_report.dur.as_micros()).unwrap()).unwrap());
        self.p99_dur_micros
            .add(f64::try_from(u32::try_from(single_report.dur.as_micros()).unwrap()).unwrap());
        match single_report.decision {
            Decision::Allow => {
                self.allows.add(1.0);
                self.denies.add(0.0);
            }
            Decision::Deny => {
                self.allows.add(0.0);
                self.denies.add(1.0);
            }
        }
        self.mean_num_errors
            .add(f64::try_from(u32::try_from(single_report.errors.len()).unwrap()).unwrap());
        match &self.err {
            Some(_) => {}
            None => {
                self.err = single_report.errors.iter().next().cloned();
            }
        }
        self.mean_context_attrs
            .add(f64::try_from(u32::try_from(single_report.context_attrs).unwrap()).unwrap());
    }

    /// Print the report's contents to the provided stream (e.g. stdout)
    pub fn print(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        writeln!(f, "mean duration: {:.3} µs", self.mean_dur_micros.mean())?;
        writeln!(
            f,
            "median duration: {:.3} µs",
            self.median_dur_micros.quantile()
        )?;
        writeln!(f, "p90 duration: {:.3} µs", self.p90_dur_micros.quantile())?;
        writeln!(f, "p99 duration: {:.3} µs", self.p99_dur_micros.quantile())?;
        writeln!(
            f,
            "{:.1}% allows and {:.1}% denies",
            100.0 * self.allows.mean(),
            100.0 * self.denies.mean()
        )?;
        writeln!(
            f,
            "mean number of context attributes: {:.3}",
            self.mean_context_attrs.mean()
        )?;
        writeln!(
            f,
            "mean number of errors per request: {:.3}",
            self.mean_num_errors.mean()
        )?;
        if let Some(err) = &self.err {
            writeln!(f, "first error encountered: {err}")?;
        }
        Ok(())
    }

    /// Print the CSV header row that corresponds to the format in
    /// `self.csv_row()` (with no newline or trailing comma)
    pub fn csv_header(f: &mut impl std::io::Write, prefix: &str) -> std::io::Result<()> {
        write!(
            f,
            "\"{prefix} mean_dur_micros\",\"{prefix} median_dur_micros\",\"{prefix} p90_dur_micros\",\"{prefix} p99_dur_micros\",\"{prefix} % allows\",\"{prefix} % denies\",\"{prefix} mean_num_errors\",\"{prefix} mean_context_attrs\""
        )?;
        Ok(())
    }

    /// Print the report's contents as a CSV row (with no newline or trailing comma)
    pub fn csv_row(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(
            f,
            "{},{},{},{},{},{},{},{}",
            self.mean_dur_micros.mean(),
            self.median_dur_micros.quantile(),
            self.p90_dur_micros.quantile(),
            self.p99_dur_micros.quantile(),
            self.allows.mean(),
            self.denies.mean(),
            self.mean_num_errors.mean(),
            self.mean_context_attrs.mean(),
        )?;
        Ok(())
    }
}

/// Aggregate stats on many generated hierarchies
pub struct HierarchyStats {
    /// Map from entity type, to mean number of parents of entities of that type
    mean_num_parents: BTreeMap<EntityType, Mean>,
    /// Mean number of `OpenFgaTuple`s (will be 0 if not doing OpenFGA)
    mean_openfga_tuples: Mean,
}

impl HierarchyStats {
    /// A new `HierarchyStats` with no data
    pub fn new() -> Self {
        Self {
            mean_num_parents: BTreeMap::new(),
            mean_openfga_tuples: Mean::new(),
        }
    }

    /// Add stats on the given hierarchy
    pub fn add(&mut self, hierarchy: &Entities, openfga_tuples: usize) {
        for entity in hierarchy.iter() {
            self.mean_num_parents
                .entry(entity.uid().entity_type().clone())
                .or_insert_with(|| Mean::new())
                .add(f64::try_from(u32::try_from(entity.ancestors().count()).unwrap()).unwrap());
        }
        self.mean_openfga_tuples
            .add(f64::try_from(u32::try_from(openfga_tuples).unwrap()).unwrap());
    }

    /// Print the CSV header row that corresponds to the format in
    /// `self.csv_row()` (with no newline or trailing comma)
    pub fn csv_header(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(
            f,
            "{}",
            self.mean_num_parents
                .keys()
                .map(|et| format!("mean_parents_of_{et}"))
                .join(",")
        )?;
        if !self.mean_num_parents.is_empty() {
            write!(f, ",")?;
        }
        write!(f, "openfga mean_tuples")?;
        Ok(())
    }

    /// Print the contents as a CSV row (with no newline or trailing comma)
    pub fn csv_row(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(
            f,
            "{}",
            self.mean_num_parents.values().map(|m| m.mean()).join(",")
        )?;
        if !self.mean_num_parents.is_empty() {
            write!(f, ",")?;
        }
        write!(f, "{}", self.mean_openfga_tuples.mean())?;
        Ok(())
    }
}

pub struct RandomBytes {
    bytes: Vec<u8>,
    rng: ThreadRng,
}

impl RandomBytes {
    /// A new `RandomBytes` instance
    pub fn new() -> Self {
        Self {
            bytes: vec![],
            rng: thread_rng(),
        }
    }

    /// Get a (new, fresh) set of random bytes with the given number of bytes,
    /// as an `Unstructured`
    pub fn unstructured(&mut self, num_bytes: usize) -> Unstructured<'_> {
        self.bytes.clear();
        self.bytes.resize_with(num_bytes, || self.rng.gen());
        Unstructured::new(&self.bytes)
    }
}
