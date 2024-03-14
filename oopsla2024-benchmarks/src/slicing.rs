//! Policy slicing functionality

use cedar_policy_core::ast::{
    EntityReference, EntityType, EntityUID, EntityUIDEntry, Policy, PolicySet, PolicySetError,
    PrincipalOrResourceConstraint, Request,
};
use cedar_policy_core::entities::{Dereference, Entities};
use itertools::Itertools;
use std::collections::HashMap;

/// Key format, which policies are indexed based on
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SliceKey {
    /// `None` is for policies with `principal,` in the scope,
    /// ie, that apply to all principals
    principal: Option<EntityUID>,
    /// `None` is for policies with `resource,` in the scope,
    /// ie, that apply to all resources
    resource: Option<EntityUID>,
}

impl SliceKey {
    /// Get the `SliceKey` under which the given `policy` should be stored
    pub fn for_policy(policy: &Policy) -> Self {
        Self {
            principal: match policy.principal_constraint().as_inner() {
                PrincipalOrResourceConstraint::Any => None,
                PrincipalOrResourceConstraint::Eq(EntityReference::EUID(uid)) => {
                    Some((**uid).clone())
                }
                PrincipalOrResourceConstraint::In(EntityReference::EUID(uid)) => {
                    Some((**uid).clone())
                }
                PrincipalOrResourceConstraint::Is(_) => None,
                PrincipalOrResourceConstraint::IsIn(_, EntityReference::EUID(uid)) => {
                    Some((**uid).clone())
                }
                PrincipalOrResourceConstraint::Eq(EntityReference::Slot)
                | PrincipalOrResourceConstraint::In(EntityReference::Slot)
                | PrincipalOrResourceConstraint::IsIn(_, EntityReference::Slot) => None,
            },
            resource: match policy.resource_constraint().as_inner() {
                PrincipalOrResourceConstraint::Any => None,
                PrincipalOrResourceConstraint::Eq(EntityReference::EUID(uid)) => {
                    Some((**uid).clone())
                }
                PrincipalOrResourceConstraint::In(EntityReference::EUID(uid)) => {
                    Some((**uid).clone())
                }
                PrincipalOrResourceConstraint::Is(_) => None,
                PrincipalOrResourceConstraint::IsIn(_, EntityReference::EUID(uid)) => {
                    Some((**uid).clone())
                }
                PrincipalOrResourceConstraint::Eq(EntityReference::Slot)
                | PrincipalOrResourceConstraint::In(EntityReference::Slot)
                | PrincipalOrResourceConstraint::IsIn(_, EntityReference::Slot) => None,
            },
        }
    }
}

/// Performs policy slicing
#[derive(Debug)]
pub struct Slicer<'p> {
    /// Policies indexed by their `SliceKey`
    indexed: HashMap<SliceKey, Vec<&'p Policy>>,
}

impl<'p> Slicer<'p> {
    /// Create a `Slicer` for the given [`PolicySet`]
    pub fn new(policy_set: &'p PolicySet) -> Self {
        let mut indexed: HashMap<SliceKey, Vec<_>> = HashMap::new();
        for policy in policy_set.policies() {
            indexed
                .entry(SliceKey::for_policy(policy))
                .or_default()
                .push(policy);
        }
        Self { indexed }
    }

    /// Get a slice of the [`PolicySet`] that contains only the policies
    /// needed for the given [`Request`].
    ///
    /// This method needs the `Entities` because it needs to know ancestors
    /// of the entities in the `Request`.
    pub fn get_slice(
        &self,
        request: &Request,
        entities: &Entities,
    ) -> Result<PolicySet, PolicySetError> {
        // We need to look up all policies for [principal and all its
        // ancestors and Any] crossed with [resource and all
        // its ancestors and Any]
        let get_all_ancestors = |uid: &EntityUID| {
            std::iter::once(Some(uid.clone()))
                .chain(std::iter::once(None))
                .chain(match entities.entity(uid) {
                    Dereference::Data(entity) => entity.ancestors().cloned().map(Some).collect(),
                    Dereference::NoSuchEntity => vec![],
                    Dereference::Residual(_) => unimplemented!("slicing with partial evaluation"),
                })
        };
        match (request.principal(), request.resource()) {
            (EntityUIDEntry::Known(principal), EntityUIDEntry::Known(resource)) => {
                match (principal.entity_type(), resource.entity_type()) {
                    (EntityType::Specified(_), EntityType::Specified(_)) => self
                        .all_policies_with_keys(
                            get_all_ancestors(principal)
                                .cartesian_product(get_all_ancestors(resource))
                                .map(|(principal, resource)| SliceKey {
                                    principal,
                                    resource,
                                }),
                        ),
                    (EntityType::Specified(_), EntityType::Unspecified) => {
                        // for requests with unspecified resource, the only matching
                        // policies are the ones that do not constrain `resource`
                        self.all_policies_with_keys(
                            get_all_ancestors(principal)
                                .cartesian_product(std::iter::once(None))
                                .map(|(principal, resource)| SliceKey {
                                    principal,
                                    resource,
                                }),
                        )
                    }
                    (EntityType::Unspecified, EntityType::Specified(_)) => {
                        // for requests with unspecified principal, the only matching
                        // policies are the ones that do not constrain `principal`
                        self.all_policies_with_keys(
                            std::iter::once(None)
                                .cartesian_product(get_all_ancestors(resource))
                                .map(|(principal, resource)| SliceKey {
                                    principal,
                                    resource,
                                }),
                        )
                    }
                    (EntityType::Unspecified, EntityType::Unspecified) => {
                        // for requests with unspecified principal _and_ resource, the
                        // only matching policies are the ones that do not constrain
                        // `principal` _or_ `resource`
                        self.all_policies_with_keys([SliceKey {
                            principal: None,
                            resource: None,
                        }])
                    }
                }
            }
            _ => unimplemented!("slicing with partial evaluation"),
        }
    }

    /// Internal utility: return a slice consisting of all the policies
    /// stored under the given `SliceKey`s
    fn all_policies_with_keys(
        &self,
        keys: impl IntoIterator<Item = SliceKey>,
    ) -> Result<PolicySet, PolicySetError> {
        let mut slice = PolicySet::new();
        for key in keys {
            if let Some(policies) = self.indexed.get(&key) {
                for &policy in policies {
                    slice.add(policy.clone())?;
                }
            }
        }
        Ok(slice)
    }
}
