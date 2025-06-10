use std::cell::RefCell;
thread_local! {
    pub static MAX_TASKS : RefCell<usize> = RefCell::default();
    pub static NUM_LISTS : RefCell<usize> = RefCell::default();
    pub static NUM_USERS : RefCell<usize> = RefCell::default();
    pub static TEAM_DEPTH : RefCell<usize> = RefCell::default();
    pub static NUM_REQUESTS : RefCell<usize> = RefCell::default();
}

pub static TINYTODO_SCHEMA_PATH: &str = "benches/tinytodo/cedar/tinytodo.cedarschema.json";
pub static TINYTODO_REGO: &str = "benches/tinytodo/rego/tinytodo.rego";
pub static TINYTODO_REGO_TCO: &str = "benches/tinytodo/rego/tinytodo_tco.rego";
pub static TRIALS: usize = 10;
