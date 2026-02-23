/*!
* Generate a markdown overview of applications and services
*/

use crate::AppState;
use crate::models::{ApplicationWithRelations, ServiceWithRelations};
use itertools::Itertools;
use std::fmt::Write;

pub trait Overview {
    fn to_md(&self, state: &AppState) -> String;
}

/// Helper: append a row to the markdown table only if the value is present
fn row(md: &mut String, label: &str, value: &str) {
    writeln!(md, "| {label} | {value} |").unwrap();
}

/// Helper: append a row to the markdown table only if the value is present
fn header(md: &mut String, label: &str, value: &str) {
    writeln!(md, "| {label} | {value} |").unwrap();
    writeln!(md, "|---------|---------|").unwrap();
}

impl Overview for ApplicationWithRelations {
    fn to_md(&self, state: &AppState) -> String {
        let mut md = String::new();

        if let Some(description) = &self.application.description {
            writeln!(md, "{description}").unwrap();
            writeln!(md).unwrap();
        }

        header(
            &mut md,
            "Auto",
            &format!("{}/{}", state.config.domain, &self.application.id[..8]),
        );

        if let Some(url) = &self.application.url {
            row(&mut md, "URL", &format!("[{url}]({url})"));
        }

        if let Some(repo) = &self.application.repository_url {
            row(&mut md, "Repo", &format!("[{repo}]({repo})"));
        }

        if !self.stacks.is_empty() {
            row(
                &mut md,
                "Stack",
                &self.stacks.iter().map(|s| &*s.name).join(", "),
            );
        }

        if !self.infra.is_empty() {
            row(
                &mut md,
                "Infra",
                &self.infra.iter().map(|i| &*i.name).join(", "),
            );
        }

        if !self.network_shares.is_empty() {
            row(
                &mut md,
                "Storage",
                &self.network_shares.iter().map(|s| &*s.name).join(", "),
            );
        }

        if !self.people.is_empty() {
            let people = self
                .people
                .iter()
                .map(|p| format!("{} ({})", p.name, p.contribution_type))
                .join(", ");
            row(&mut md, "People", &people);
        }

        md
    }
}

impl Overview for ServiceWithRelations {
    fn to_md(&self, state: &AppState) -> String {
        let mut md = String::new();

        if let Some(description) = &self.service.description {
            writeln!(md, "{description}").unwrap();
            writeln!(md).unwrap();
        }

        header(
            &mut md,
            "Auto",
            &format!("{}/{}", state.config.domain, &self.service.id[..8]),
        );

        if let Some(repo) = &self.service.repository_url {
            row(&mut md, "Repo", &format!("[{repo}]({repo})"));
        }

        if !self.infra.is_empty() {
            row(
                &mut md,
                "Infra",
                &self.infra.iter().map(|i| &*i.name).join(", "),
            );
        }

        md
    }
}
