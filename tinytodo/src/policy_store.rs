/*
 * Copyright 2022-2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::fmt::Display;

use cedar_policy::{
    ParseErrors, Policy, PolicyId, PolicySet, PolicySetError, SchemaError, ValidationError,
    Validator,
};
use thiserror::Error;

#[derive(Debug)]
pub struct PolicyStore {
    policy_set: PolicySet,
    dynamic_policies: Vec<Policy>,
    validator: Validator,
    id_counter: usize,
}

impl PolicyStore {
    pub fn new(policy_src: &str, schema_src: &str) -> Result<Self> {
        let policy_set = policy_src.parse()?;
        let schema = schema_src.parse()?;
        let validator = Validator::new(schema);
        let results = validator.validate(&policy_set, cedar_policy::ValidationMode::Strict);
        if results.validation_passed() {
            Ok(Self {
                policy_set,
                validator,
                dynamic_policies: vec![],
                id_counter: 0,
            })
        } else {
            Err(Error::validation(results.validation_errors()))
        }
    }

    pub fn fresh_policy_id(&mut self) -> PolicyId {
        loop {
            let id_try: PolicyId = format!("Dynamic-Policy-{}", self.id_counter)
                .parse()
                .unwrap();
            self.id_counter += 1;
            if self.policy_set.policy(&id_try).is_none() {
                return id_try;
            }
        }
    }

    pub fn policies(&self) -> &PolicySet {
        &self.policy_set
    }

    pub fn set_static_store(&mut self, p: PolicySet) -> Result<()> {
        self.policy_set = p;
        for dynamic in self.dynamic_policies.iter() {
            self.policy_set.add(dynamic.clone())?;
        }
        Ok(())
    }

    pub fn add_dynamic_policy(&mut self, p: Policy) -> Result<()> {
        self.validate_policy(p.clone())?;
        self.policy_set.add(p.clone())?;
        self.dynamic_policies.push(p);
        Ok(())
    }

    fn validate_policy(&self, p: Policy) -> Result<()> {
        let singleton = PolicySet::from_policies([p]).unwrap();
        let results = self
            .validator
            .validate(&singleton, cedar_policy::ValidationMode::Strict);
        if results.validation_passed() {
            Ok(())
        } else {
            Err(Error::validation(results.validation_errors()))
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error parsing policies: {0}")]
    PolicyParse(#[from] ParseErrors),
    #[error("Error parsing schema: {0}")]
    SchemaParse(#[from] SchemaError),
    #[error("Error validating policies: {0}")]
    Validation(String),
    #[error("Error adding policy to set: {0}")]
    Duplicate(#[from] PolicySetError),
}

impl Error {
    pub fn validation<'a>(errs: impl Iterator<Item = &'a ValidationError<'a>>) -> Self {
        Self::Validation(ValidationErrors(errs.collect()).to_string())
    }
}

#[derive(Debug)]
pub struct ValidationErrors<'a>(pub Vec<&'a ValidationError<'a>>);

impl<'a> Display for ValidationErrors<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in self.0.iter() {
            writeln!(f, "{}", err)?;
        }
        Ok(())
    }
}
