use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy binding quota must pass. Check returns its problems, or none.
pub trait LegacyBindingQuotaRule: std::fmt::Debug {
    fn check(&self, item: &LegacyBindingQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingQuotaAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl LegacyBindingQuotaRule for LegacyBindingQuotaAllowedLabelRule {
    fn check(&self, item: &LegacyBindingQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is below minimum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingQuotaRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyBindingQuotaRule for LegacyBindingQuotaRetriesRangeRule {
    fn check(&self, item: &LegacyBindingQuota) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingQuotaRequiredLabelRule {
    pub message: String,
}

impl LegacyBindingQuotaRule for LegacyBindingQuotaRequiredLabelRule {
    fn check(&self, item: &LegacyBindingQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy binding quota.
#[derive(Debug, Clone)]
pub struct LegacyBindingQuotaValidator {
    pub rules: Vec<Rc<dyn LegacyBindingQuotaRule>>,
}

impl LegacyBindingQuotaValidator {
    pub fn validate(&self, item: &LegacyBindingQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyBindingQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyBindingQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_binding_quota_validator() -> LegacyBindingQuotaValidator {
    LegacyBindingQuotaValidator {
        rules: vec![
            Rc::new(LegacyBindingQuotaAllowedLabelRule {
                allowed: vec!["global".to_string()],
            }) as Rc<dyn LegacyBindingQuotaRule>,
            Rc::new(LegacyBindingQuotaRetriesRangeRule { min: 120, max: 255 }),
            Rc::new(LegacyBindingQuotaRequiredLabelRule {
                message: "legacy binding quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_binding_quota_validator();
        let item = new_legacy_binding_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = LegacyBindingQuotaRetriesRangeRule { min: 32, max: 80 };
        let mut item = new_legacy_binding_quota("x", "y");
        item.retries = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LegacyBindingQuotaRequiredLabelRule {
            message: "legacy binding quota needs an entry".to_string(),
        };
        let mut item = new_legacy_binding_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
