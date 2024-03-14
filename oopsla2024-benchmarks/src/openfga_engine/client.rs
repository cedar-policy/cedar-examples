use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::time::Duration;

pub struct OpenFgaClient {
    /// Underlying `reqwest` Client
    client: reqwest::blocking::Client,
    /// OpenFGA server URL (including protocol and port)
    url: String,
    /// OpenFGA store id
    store_id: String,
    /// OpenFGA authorization model id
    authz_model: String,
    /// Count of how many `OpenFgaTuple`s have been added via this client
    num_tuples: usize,
}

#[allow(dead_code)] // we implement more OpenFGA methods than we actually use
impl OpenFgaClient {
    /// url: the OpenFGA server URL (including protocol and port)
    ///
    /// authz_model: the OpenFGA authorization model filename
    pub fn new(url: impl Into<String>, authz_model_filename: impl AsRef<Path>) -> Self {
        let url = url.into();
        let client = reqwest::blocking::Client::new();

        let store_id = {
            let resp = match client
                .post(format!("{url}/stores"))
                .json(&json!({ "name": "openfga-app" }))
                .send()
            {
                Ok(resp) => resp,
                Err(_) => {
                    // wait 20ms and retry once
                    std::thread::sleep(Duration::from_millis(20));
                    client
                        .post(format!("{url}/stores"))
                        .json(&json!({ "name": "openfga-app" }))
                        .send()
                        .expect("failed to create OpenFGA store, even on retry")
                }
            };
            if let Err(e) = resp.error_for_status_ref() {
                panic!(
                    "failed to create OpenFGA store: {e} -- {}",
                    resp.text().unwrap()
                );
            }
            resp.json::<HashMap<String, String>>()
                .expect("successful response contained bad json")
                .get("id")
                .expect("no 'id' field in response")
                .clone()
        };

        let authz_model = {
            let file = File::open(authz_model_filename)
                .expect("failed to open OpenFGA authorization model");
            let resp = client
                .post(format!("{url}/stores/{store_id}/authorization-models"))
                .body(file)
                .send()
                .expect("failed to create OpenFGA authorization model");
            if let Err(e) = resp.error_for_status_ref() {
                panic!(
                    "failed to create OpenFGA authorization model: {e} -- {}",
                    resp.text().unwrap()
                )
            }
            resp.json::<HashMap<String, String>>()
                .expect("successful response contained bad json")
                .get("authorization_model_id")
                .expect("no 'authorization_model_id' field in response")
                .clone()
        };

        Self {
            client,
            url,
            store_id,
            authz_model,
            num_tuples: 0,
        }
    }

    /// OpenFGA "Get a store" command
    ///
    /// Panics on failure
    pub fn get_store(&self) -> reqwest::blocking::Response {
        let resp = self
            .client
            .get(format!("{}/stores/{}", self.url, self.store_id))
            .send()
            .expect("failed to send request");
        if let Err(e) = resp.error_for_status_ref() {
            panic!("failed to get_store: {e} -- {}", resp.text().unwrap());
        }
        resp
    }

    /// OpenFGA "Get tuples" command
    ///
    /// Accepts a query filter, but for now we only expose the version that returns all tuples
    ///
    /// Panics on failure
    pub fn read_tuples(&self) -> impl Iterator<Item = OpenFgaTuple> {
        let resp = self
            .client
            .post(format!("{}/stores/{}/read", self.url, self.store_id))
            .json(&json!({}))
            .send()
            .expect("failed to send request");
        if let Err(e) = resp.error_for_status_ref() {
            panic!("failed to read_tuples: {e} -- {}", resp.text().unwrap());
        }
        resp.json::<ReadTuplesPayload>()
            .expect("successful response contained bad json")
            .tuples
            .into_iter()
            .map(|tuple| tuple.key)
    }

    /// OpenFGA "Add or delete tuples" command
    ///
    /// Panics on failure
    ///
    /// This OpenFGA API doesn't return anything in the success case, so this function doesn't return anything
    pub fn write_tuples(
        &mut self,
        adds: impl IntoIterator<Item = OpenFgaTuple>,
        deletes: impl IntoIterator<Item = OpenFgaTuple>,
    ) {
        let add_tuple_keys = TupleKeys::new(adds);
        self.num_tuples += add_tuple_keys.len();
        let delete_tuple_keys = TupleKeys::new(deletes);
        if add_tuple_keys.is_empty() && delete_tuple_keys.is_empty() {
            // OpenFGA will reject a request with neither adds nor deletes
            warn!("ignoring call to write_tuples with no tuples to add or delete");
            return;
        }
        let write_batch = |adds, deletes| {
            let resp = self
                .client
                .post(format!("{}/stores/{}/write", self.url, self.store_id))
                .json(&WriteTuplesPayload::new(adds, deletes, &self.authz_model))
                .send()
                .expect("failed to send request");
            if let Err(e) = resp.error_for_status_ref() {
                panic!("failed to write_tuples: {e} -- {}", resp.text().unwrap());
            }
        };
        if add_tuple_keys.len() + delete_tuple_keys.len() <= 100 {
            // do it all in one batch
            write_batch(add_tuple_keys, delete_tuple_keys);
        } else {
            // OpenFGA takes at most 100 tuples at once, so we need to split into batches of 100 or less
            for add_batch in add_tuple_keys.batches(100) {
                write_batch(add_batch, TupleKeys::empty());
            }
            for delete_batch in delete_tuple_keys.batches(100) {
                write_batch(TupleKeys::empty(), delete_batch);
            }
        }
    }

    /// Shorthand for adding tuples
    ///
    /// Panics on failure
    ///
    /// This OpenFGA API doesn't return anything in the success case, so this function doesn't return anything
    pub fn add_tuples(&mut self, adds: impl IntoIterator<Item = OpenFgaTuple>) {
        self.write_tuples(adds, [])
    }

    /// Shorthand for deleting tuples
    ///
    /// Panics on failure
    ///
    /// This OpenFGA API doesn't return anything in the success case, so this function doesn't return anything
    pub fn delete_tuples(&mut self, deletes: impl IntoIterator<Item = OpenFgaTuple>) {
        self.write_tuples([], deletes)
    }

    /// OpenFGA "check" command
    ///
    /// Returns `true` if the access is allowed
    ///
    /// Panics on failure
    pub fn check(&self, tuple: &OpenFgaTuple) -> bool {
        let resp = self
            .client
            .post(format!("{}/stores/{}/check", self.url, self.store_id))
            .json(&CheckPayload {
                tuple_key: tuple,
                authorization_model_id: &self.authz_model,
            })
            .send()
            .expect("failed to send request");
        if let Err(e) = resp.error_for_status_ref() {
            panic!("failed to check: {e} -- {}", resp.text().unwrap());
        }
        resp.json::<Map<String, serde_json::Value>>()
            .expect("successful response contained bad json")
            .get("allowed")
            .expect("no 'allowed' field in response")
            .as_bool()
            .expect("'allowed' field was not a boolean")
    }

    /// How many `OpenFgaTuple`s have been added via this client
    pub fn num_tuples(&self) -> usize {
        self.num_tuples
    }
}

/// Represents a "tuple", ie an OpenFGA relationship
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct OpenFgaTuple {
    pub user: String,
    pub relation: String,
    pub object: String,
}

/// JSON payload for the `write_tuples` command
#[derive(Debug, PartialEq, Eq, Serialize)]
struct WriteTuplesPayload<'a> {
    #[serde(skip_serializing_if = "TupleKeys::is_empty")]
    writes: TupleKeys,
    #[serde(skip_serializing_if = "TupleKeys::is_empty")]
    deletes: TupleKeys,
    authorization_model_id: &'a str,
}

impl<'a> WriteTuplesPayload<'a> {
    fn new(writes: TupleKeys, deletes: TupleKeys, authorization_model_id: &'a str) -> Self {
        Self {
            writes,
            deletes,
            authorization_model_id,
        }
    }
}

/// Part of the `WriteTuplesPayload` structure
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct TupleKeys {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    tuple_keys: Vec<OpenFgaTuple>,
}

impl TupleKeys {
    fn new(tuples: impl IntoIterator<Item = OpenFgaTuple>) -> Self {
        Self {
            tuple_keys: tuples.into_iter().collect(),
        }
    }

    fn empty() -> Self {
        Self {
            tuple_keys: Vec::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.tuple_keys.is_empty()
    }

    fn len(&self) -> usize {
        self.tuple_keys.len()
    }

    // TODO: do this without cloning, probably means `TupleKeys` needs to not own a `Vec`
    fn batches(&self, batch_size: usize) -> Box<dyn Iterator<Item = Self> + '_> {
        if self.is_empty() {
            Box::new(std::iter::empty())
        } else if self.len() <= batch_size {
            Box::new(std::iter::once(self.clone()))
        } else {
            Box::new(
                self.tuple_keys
                    .chunks(batch_size)
                    .map(|chunk| TupleKeys::new(chunk.iter().cloned())),
            )
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct ReadTuplesPayload {
    tuples: Vec<TupleWithTimestamp>,
    continuation_token: String,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct TupleWithTimestamp {
    key: OpenFgaTuple,
    timestamp: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct CheckPayload<'a> {
    tuple_key: &'a OpenFgaTuple,
    authorization_model_id: &'a str,
}
