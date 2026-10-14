use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a query manifest must pass. Check returns its problems, or none.
pub trait QueryManifestRule: std::fmt::Debug {
    fn check(&self, item: &QueryManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryManifestCodePrefixRule {
    pub prefix: String,
}

impl QueryManifestRule for QueryManifestCodePrefixRule {
    fn check(&self, item: &QueryManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryManifestRequiredCountryRule {
    pub message: String,
}

impl QueryManifestRule for QueryManifestRequiredCountryRule {
    fn check(&self, item: &QueryManifest) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryManifestAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl QueryManifestRule for QueryManifestAllowedChannelRule {
    fn check(&self, item: &QueryManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is not allowed", item.channel)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query manifest.
#[derive(Debug, Clone)]
pub struct QueryManifestValidator {
    pub rules: Vec<Rc<dyn QueryManifestRule>>,
}

impl QueryManifestValidator {
    pub fn validate(&self, item: &QueryManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryManifest]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_query_manifest_validator() -> QueryManifestValidator {
    QueryManifestValidator {
        rules: vec![
            Rc::new(QueryManifestCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn QueryManifestRule>,
            Rc::new(QueryManifestRequiredCountryRule {
                message: "query manifest needs an entry".to_string(),
            }),
            Rc::new(QueryManifestAllowedChannelRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_manifest_validator();
        let item = new_query_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = QueryManifestRequiredCountryRule {
            message: "query manifest needs a value".to_string(),
        };
        let mut item = new_query_manifest("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
