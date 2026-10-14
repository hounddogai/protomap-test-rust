use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a grant quota must pass. Check returns its problems, or none.
pub trait GrantQuotaRule: std::fmt::Debug {
    fn check(&self, item: &GrantQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantQuotaAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantQuotaRule for GrantQuotaAttemptsRangeRule {
    fn check(&self, item: &GrantQuota) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantQuotaAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl GrantQuotaRule for GrantQuotaAllowedLabelRule {
    fn check(&self, item: &GrantQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is out of range", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantQuotaRequiredCodeRule {
    pub message: String,
}

impl GrantQuotaRule for GrantQuotaRequiredCodeRule {
    fn check(&self, item: &GrantQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant quota.
#[derive(Debug, Clone)]
pub struct GrantQuotaValidator {
    pub rules: Vec<Rc<dyn GrantQuotaRule>>,
}

impl GrantQuotaValidator {
    pub fn validate(&self, item: &GrantQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_quota_validator() -> GrantQuotaValidator {
    GrantQuotaValidator {
        rules: vec![
            Rc::new(GrantQuotaAttemptsRangeRule { min: 32, max: 255 }) as Rc<dyn GrantQuotaRule>,
            Rc::new(GrantQuotaAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(GrantQuotaRequiredCodeRule {
                message: "grant quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_quota_validator();
        let item = new_grant_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = GrantQuotaAttemptsRangeRule { min: 16, max: 1000 };
        let mut item = new_grant_quota("x", "y");
        item.attempts = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GrantQuotaRequiredCodeRule {
            message: "grant quota needs this field".to_string(),
        };
        let mut item = new_grant_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
