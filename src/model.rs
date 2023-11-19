use std::fmt::{Display, Formatter, Result};

/// Represent a crate version
#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    id: u64,
    pub num: String,
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.num)
    }
}

/// Represent a crate
#[derive(Debug, Serialize, Deserialize)]
pub struct Crate {
    pub created_at: String,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub downloads: u64,
    pub homepage: Option<String>,
    pub id: String,
    pub keywords: Vec<String>,
    pub license: Option<String>,
    pub max_version: String,
    pub name: String,
    pub repository: Option<String>,
    pub updated_at: String,
}

/// crate metadata to print
#[derive(Debug, Serialize, Deserialize)]
pub struct CrateMetadata {
    // in response.crate
    #[serde(rename = "crate")]
    pub crate_data: Crate,
    pub versions: Vec<Version>,
}
