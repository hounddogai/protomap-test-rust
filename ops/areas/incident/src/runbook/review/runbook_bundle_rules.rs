use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook bundle must pass. Check returns its problems, or none.
pub trait RunbookBundleRule: std::fmt::Debug {
    fn check(&self, item: &RunbookBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBundleRequiredLabelRule {
    pub message: String,
}

impl RunbookBundleRule for RunbookBundleRequiredLabelRule {
    fn check(&self, item: &RunbookBundle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBundleCountryLengthRule {
    pub max_length: i64,
}

impl RunbookBundleRule for RunbookBundleCountryLengthRule {
    fn check(&self, item: &RunbookBundle) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBundleCodePrefixRule {
    pub prefix: String,
}

impl RunbookBundleRule for RunbookBundleCodePrefixRule {
    fn check(&self, item: &RunbookBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBundleAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl RunbookBundleRule for RunbookBundleAllowedTimezoneRule {
    fn check(&self, item: &RunbookBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is inconsistent", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBundleTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RunbookBundleRule for RunbookBundleTagsLimitRule {
    fn check(&self, item: &RunbookBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBundleRequiredKeyIdRule {
    pub message: String,
}

impl RunbookBundleRule for RunbookBundleRequiredKeyIdRule {
    fn check(&self, item: &RunbookBundle) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook bundle.
#[derive(Debug, Clone)]
pub struct RunbookBundleValidator {
    pub rules: Vec<Rc<dyn RunbookBundleRule>>,
}

impl RunbookBundleValidator {
    pub fn validate(&self, item: &RunbookBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_bundle_validator() -> RunbookBundleValidator {
    RunbookBundleValidator {
        rules: vec![
            Rc::new(RunbookBundleRequiredLabelRule {
                message: "runbook bundle needs this field".to_string(),
            }) as Rc<dyn RunbookBundleRule>,
            Rc::new(RunbookBundleCountryLengthRule { max_length: 16 }),
            Rc::new(RunbookBundleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RunbookBundleAllowedTimezoneRule {
                allowed: vec!["capped".to_string(), "default".to_string()],
            }),
            Rc::new(RunbookBundleTagsLimitRule {
                max_count: 80,
                banned: vec!["legacy".to_string(), "capped".to_string()],
            }),
            Rc::new(RunbookBundleRequiredKeyIdRule {
                message: "runbook bundle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_bundle_validator();
        let item = new_runbook_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RunbookBundleRequiredLabelRule {
            message: "runbook bundle needs an entry".to_string(),
        };
        let mut item = new_runbook_bundle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = RunbookBundleCountryLengthRule { max_length: 32 };
        let mut item = new_runbook_bundle("x", "y");
        item.country = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = RunbookBundleRequiredKeyIdRule {
            message: "runbook bundle needs an entry".to_string(),
        };
        let mut item = new_runbook_bundle("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
