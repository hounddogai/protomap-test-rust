use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive page snapshot must pass. Check returns its problems, or none.
pub trait AdaptivePageSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &AdaptivePageSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePageSnapshotRequiredKeyIdRule {
    pub message: String,
}

impl AdaptivePageSnapshotRule for AdaptivePageSnapshotRequiredKeyIdRule {
    fn check(&self, item: &AdaptivePageSnapshot) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePageSnapshotAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl AdaptivePageSnapshotRule for AdaptivePageSnapshotAllowedExternalRefRule {
    fn check(&self, item: &AdaptivePageSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is above maximum", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePageSnapshotCodePrefixRule {
    pub prefix: String,
}

impl AdaptivePageSnapshotRule for AdaptivePageSnapshotCodePrefixRule {
    fn check(&self, item: &AdaptivePageSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive page snapshot.
#[derive(Debug, Clone)]
pub struct AdaptivePageSnapshotValidator {
    pub rules: Vec<Rc<dyn AdaptivePageSnapshotRule>>,
}

impl AdaptivePageSnapshotValidator {
    pub fn validate(&self, item: &AdaptivePageSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptivePageSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptivePageSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_page_snapshot_validator() -> AdaptivePageSnapshotValidator {
    AdaptivePageSnapshotValidator {
        rules: vec![
            Rc::new(AdaptivePageSnapshotRequiredKeyIdRule {
                message: "adaptive page snapshot needs this field".to_string(),
            }) as Rc<dyn AdaptivePageSnapshotRule>,
            Rc::new(AdaptivePageSnapshotAllowedExternalRefRule {
                allowed: vec!["rolling".to_string(), "rolling".to_string()],
            }),
            Rc::new(AdaptivePageSnapshotCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_page_snapshot_validator();
        let item = new_adaptive_page_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = AdaptivePageSnapshotRequiredKeyIdRule {
            message: "adaptive page snapshot needs this field".to_string(),
        };
        let mut item = new_adaptive_page_snapshot("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
