//! `tyr` context
use mimir;

/// `tyr` context
#[derive(Builder, Debug, Getters)]
pub struct Context {
    /// `mimir` context
    #[get = "pub"]
    #[builder(default = "self.default_db_context()?")]
    db_context: mimir::Context,
    /// Connection string.
    #[get = "pub"]
    conn_string: String,
    /// Username use for db connection.
    #[get = "pub"]
    username: String,
    /// Password used for db connection.
    #[get = "pub"]
    password: String,
}

impl ContextBuilder {
    /// Generate the default db context.
    fn default_db_context(&self) -> Result<mimir::Context, String> {
        Ok(mimir::ContextBuilder::default().build()?)
    }
}
