//! `tyr` environment
use error::Result;
use std::collections::HashMap;

const COMMON_PREFIX: &'static str = "common";
const DEV_PREFIX: &'static str = "dev";
const TEST_PREFIX: &'static str = "test";
const PROD_PREFIX: &'static str = "prod";
const ENV_SUFFIX: &'static str = ".env";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Kind {
    Dev,
    Test,
    Prod,
}

pub struct Common {
    kvs: HashMap<String, String>,
}

pub struct Specific {
    kind: Kind,
    kvs: HashMap<String, String>,
}
