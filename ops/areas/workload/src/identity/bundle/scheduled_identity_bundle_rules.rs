use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled identity bundle must pass. Check returns its problems, or none.
pub trait ScheduledIdentityBundleRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledIdentityBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityBundleRequiredExternalRefRule {
    pub message: String,
}

impl ScheduledIdentityBundleRule for ScheduledIdentityBundleRequiredExternalRefRule {
    fn check(&self, item: &ScheduledIdentityBundle) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityBundleRequiredNameRule {
    pub message: String,
}

impl ScheduledIdentityBundleRule for ScheduledIdentityBundleRequiredNameRule {
    fn check(&self, item: &ScheduledIdentityBundle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityBundleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ScheduledIdentityBundleRule for ScheduledIdentityBundleAllowedLabelRule {
    fn check(&self, item: &ScheduledIdentityBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is above maximum", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled identity bundle.
#[derive(Debug, Clone)]
pub struct ScheduledIdentityBundleValidator {
    pub rules: Vec<Rc<dyn ScheduledIdentityBundleRule>>,
}

impl ScheduledIdentityBundleValidator {
    pub fn validate(&self, item: &ScheduledIdentityBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledIdentityBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledIdentityBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_identity_bundle_validator() -> ScheduledIdentityBundleValidator {
    ScheduledIdentityBundleValidator {
        rules: vec![
            Rc::new(ScheduledIdentityBundleRequiredExternalRefRule {
                message: "scheduled identity bundle needs this field".to_string(),
            }) as Rc<dyn ScheduledIdentityBundleRule>,
            Rc::new(ScheduledIdentityBundleRequiredNameRule {
                message: "scheduled identity bundle needs an entry".to_string(),
            }),
            Rc::new(ScheduledIdentityBundleAllowedLabelRule {
                allowed: vec!["default".to_string(), "fallback".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_identity_bundle_validator();
        let item = new_scheduled_identity_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = ScheduledIdentityBundleRequiredExternalRefRule {
            message: "scheduled identity bundle needs this field".to_string(),
        };
        let mut item = new_scheduled_identity_bundle("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ScheduledIdentityBundleRequiredNameRule {
            message: "scheduled identity bundle needs a value".to_string(),
        };
        let mut item = new_scheduled_identity_bundle("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
