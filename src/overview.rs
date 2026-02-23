/*!
* Generate a markdown overview of applications and services
*/

use crate::AppState;
use itertools::Itertools;
use std::fmt::{Display, Write};
use url::Url;

#[derive(Debug, Clone)]
pub struct Person {
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct Overview {
    /// The UUID of this component, in auto
    pub auto_id: String,
    /// Short description of the component
    pub description: Option<String>,
    /// URL where this application / service is accessible
    pub url: Option<Url>,
    /// URL of the repository where the source is hosted
    pub repo: Option<Url>,
    /// Technologies that are used in this app
    pub stack: Vec<String>,
    /// Infrastructure that this component relies on
    pub infra: Vec<String>,
    /// Storage pools used for this component
    pub storage: Vec<String>,
    /// People working on this project and their role
    pub people: Vec<Person>,
}

impl Display for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.role)
    }
}

impl Overview {
    /// Create a Markdown string of this overview
    pub fn to_md(&self, state: &AppState) -> String {
        let mut md = String::new();

        if let Some(description) = &self.description {
            writeln!(md, "{description}").unwrap();
            writeln!(md).unwrap();
        }

        // header: link to Auto for full information
        writeln!(md, "| Auto | {}/{} |", state.config.domain, self.auto_id).unwrap();

        if let Some(url) = &self.url {
            writeln!(md, "| URL | [{0}]({0}) |", url).unwrap();
        }

        if let Some(repo) = &self.repo {
            writeln!(md, "| Repo | [{0}]({0}) |", repo).unwrap();
        }

        if !self.stack.is_empty() {
            let cs = self.stack.join(", ");
            writeln!(md, "| Stack | {cs} |").unwrap();
        }

        if !self.infra.is_empty() {
            let cs = self.infra.join(", ");
            writeln!(md, "| Infra | {cs} |").unwrap();
        }

        if !self.storage.is_empty() {
            let cs = self.storage.join(", ");
            writeln!(md, "| Storage | {cs} |").unwrap();
        }

        if !self.people.is_empty() {
            let cs = self.people.iter().join(", ");
            writeln!(md, "| People | {cs} |").unwrap();
        }

        md
    }
}
