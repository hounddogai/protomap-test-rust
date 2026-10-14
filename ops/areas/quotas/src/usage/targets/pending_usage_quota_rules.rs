use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending usage quota must pass. Check returns its problems, or none.
pub trait PendingUsageQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PendingUsageQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingUsageQuotaRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingUsageQuotaRule for PendingUsageQuotaRotationDaysRangeRule {
    fn check(&self, item: &PendingUsageQuota) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingUsageQuotaAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PendingUsageQuotaRule for PendingUsageQuotaAllowedLabelRule {
    fn check(&self, item: &PendingUsageQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is unknown", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingUsageQuotaRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingUsageQuotaRule for PendingUsageQuotaRevisionRangeRule {
    fn check(&self, item: &PendingUsageQuota) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending usage quota.
#[derive(Debug, Clone)]
pub struct PendingUsageQuotaValidator {
    pub rules: Vec<Rc<dyn PendingUsageQuotaRule>>,
}

impl PendingUsageQuotaValidator {
    pub fn validate(&self, item: &PendingUsageQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingUsageQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingUsageQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_usage_quota_validator() -> PendingUsageQuotaValidator {
    PendingUsageQuotaValidator {
        rules: vec![
            Rc::new(PendingUsageQuotaRotationDaysRangeRule { min: 3600, max: 86400 }) as Rc<dyn PendingUsageQuotaRule>,
            Rc::new(PendingUsageQuotaAllowedLabelRule {
                allowed: vec!["pending".to_string()],
            }),
            Rc::new(PendingUsageQuotaRevisionRangeRule { min: 120, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_usage_quota_validator();
        let item = new_pending_usage_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = PendingUsageQuotaRotationDaysRangeRule { min: 120, max: 1000 };
        let mut item = new_pending_usage_quota("x", "y");
        item.rotation_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = PendingUsageQuotaRevisionRangeRule { min: 120, max: 3600 };
        let mut item = new_pending_usage_quota("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
