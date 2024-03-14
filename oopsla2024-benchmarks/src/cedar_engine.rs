use super::{ExampleApp, SingleExecutionReport, TemplateLink};
use crate::slicing::Slicer;
use cedar_policy_core::{
    ast::{Entity, PolicyID, PolicySet, Request, SlotId},
    authorizer::Authorizer,
    entities::{Entities, TCComputation},
    extensions::Extensions,
};
use cedar_policy_validator::CoreSchema;
use std::time::Instant;

pub struct CedarEngine {
    /// hierarchy object, as Cedar `Entities`
    entities: Entities,
    /// authorizer object
    authorizer: Authorizer,
    /// Cedar policies
    policies: PolicySet,
}

impl CedarEngine {
    pub fn new<'a, 'b>(
        entities: impl IntoIterator<Item = &'a Entity>,
        links: impl IntoIterator<Item = &'b TemplateLink>,
        app: &ExampleApp,
    ) -> Self {
        let mut pset = app.static_policies.clone();
        for link in links {
            let link_id = PolicyID::from_string(format!(
                "{}:{}:{}",
                link.template_name, link.principal, link.resource
            ));
            pset.link(
                PolicyID::from_string(&link.template_name),
                link_id,
                [
                    (SlotId::principal(), link.principal.clone()),
                    (SlotId::resource(), link.resource.clone()),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();
        }
        Self {
            entities: Entities::from_entities(
                entities.into_iter().cloned(),
                Some(&CoreSchema::new(&app.validator_schema())),
                TCComputation::ComputeNow,
                Extensions::all_available(),
            )
            .expect("failed to construct Entities"),
            authorizer: Authorizer::new(),
            policies: pset,
        }
    }

    pub fn execute(&self, request: Request) -> SingleExecutionReport {
        let num_context_attrs = request
            .context()
            .map(|ctx| ctx.iter().map(|it| it.count()).unwrap_or(0))
            .unwrap_or(0);
        let start = Instant::now();
        let response = self
            .authorizer
            .is_authorized(request, &self.policies, &self.entities);
        let dur = start.elapsed();
        SingleExecutionReport {
            dur,
            decision: response.decision,
            errors: response.diagnostics.errors,
            context_attrs: num_context_attrs,
        }
    }
}

pub struct CedarOptEngine {
    /// hierarchy object, as Cedar `Entities`
    entities: Entities,
    /// authorizer object
    authorizer: Authorizer,
    /// Cedar policies
    policies: PolicySet,
}

impl CedarOptEngine {
    pub fn new<'a, 'b>(
        entities: impl IntoIterator<Item = &'a Entity>,
        links: impl IntoIterator<Item = &'b TemplateLink>,
        app: &ExampleApp,
    ) -> Self {
        let mut pset = app.static_policies.clone();
        for link in links {
            let link_id = PolicyID::from_string(format!(
                "{}:{}:{}",
                link.template_name, link.principal, link.resource
            ));
            pset.link(
                PolicyID::from_string(&link.template_name),
                link_id,
                [
                    (SlotId::principal(), link.principal.clone()),
                    (SlotId::resource(), link.resource.clone()),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();
        }
        let entities = Entities::from_entities(
            entities.into_iter().cloned(),
            Some(&CoreSchema::new(&app.validator_schema())),
            TCComputation::ComputeNow,
            Extensions::all_available(),
        )
        .expect("failed to convert hierarchy to Entities");
        Self {
            entities,
            authorizer: Authorizer::new(),
            policies: pset,
        }
    }

    pub fn get_slicer(&self) -> Slicer {
        crate::slicing::Slicer::new(&self.policies)
    }

    pub fn execute(&self, request: Request, slicer: &Slicer) -> SingleExecutionReport {
        let num_context_attrs = request
            .context()
            .map(|ctx| ctx.iter().map(|it| it.count()).unwrap_or(0))
            .unwrap_or(0);
        let start = Instant::now();
        let sliced = slicer
            .get_slice(&request, &self.entities)
            .expect("failed to slice policies");
        let response = self
            .authorizer
            .is_authorized(request, &sliced, &self.entities);
        let dur = start.elapsed();
        SingleExecutionReport {
            dur,
            decision: response.decision,
            errors: response.diagnostics.errors,
            context_attrs: num_context_attrs,
        }
    }
}
