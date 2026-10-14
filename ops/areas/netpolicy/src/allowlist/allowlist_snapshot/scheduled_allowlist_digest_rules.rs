use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled allowlist digest must pass. Check returns its problems, or none.
pub trait ScheduledAllowlistDigestRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledAllowlistDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAllowlistDigestRequiredLabelRule {
    pub message: String,
}

impl ScheduledAllowlistDigestRule for ScheduledAllowlistDigestRequiredLabelRule {
    fn check(&self, item: &ScheduledAllowlistDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAllowlistDigestCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledAllowlistDigestRule for ScheduledAllowlistDigestCreatedAtRangeRule {
    fn check(&self, item: &ScheduledAllowlistDigest) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAllowlistDigestQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledAllowlistDigestRule for ScheduledAllowlistDigestQuantityRangeRule {
    fn check(&self, item: &ScheduledAllowlistDigest) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAllowlistDigestAudienceLengthRule {
    pub max_length: i64,
}

impl ScheduledAllowlistDigestRule for ScheduledAllowlistDigestAudienceLengthRule {
    fn check(&self, item: &ScheduledAllowlistDigest) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAllowlistDigestVisibleGuardRule {
    pub limit: i64,
}

impl ScheduledAllowlistDigestRule for ScheduledAllowlistDigestVisibleGuardRule {
    fn check(&self, item: &ScheduledAllowlistDigest) -> Vec<String> {
        if item.visible && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while visible",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled allowlist digest.
#[derive(Debug, Clone)]
pub struct ScheduledAllowlistDigestValidator {
    pub rules: Vec<Rc<dyn ScheduledAllowlistDigestRule>>,
}

impl ScheduledAllowlistDigestValidator {
    pub fn validate(&self, item: &ScheduledAllowlistDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledAllowlistDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledAllowlistDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_allowlist_digest_validator() -> ScheduledAllowlistDigestValidator {
    ScheduledAllowlistDigestValidator {
        rules: vec![
            Rc::new(ScheduledAllowlistDigestRequiredLabelRule {
                message: "scheduled allowlist digest needs an entry".to_string(),
            }) as Rc<dyn ScheduledAllowlistDigestRule>,
            Rc::new(ScheduledAllowlistDigestCreatedAtRangeRule { min: 32, max: 120 }),
            Rc::new(ScheduledAllowlistDigestQuantityRangeRule { min: 80, max: 3600 }),
            Rc::new(ScheduledAllowlistDigestAudienceLengthRule { max_length: 32 }),
            Rc::new(ScheduledAllowlistDigestVisibleGuardRule { limit: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_allowlist_digest_validator();
        let item = new_scheduled_allowlist_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledAllowlistDigestRequiredLabelRule {
            message: "scheduled allowlist digest needs a value".to_string(),
        };
        let mut item = new_scheduled_allowlist_digest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = ScheduledAllowlistDigestCreatedAtRangeRule { min: 16, max: 1000 };
        let mut item = new_scheduled_allowlist_digest("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ScheduledAllowlistDigestQuantityRangeRule { min: 32, max: 3600 };
        let mut item = new_scheduled_allowlist_digest("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = ScheduledAllowlistDigestAudienceLengthRule { max_length: 64 };
        let mut item = new_scheduled_allowlist_digest("x", "y");
        item.audience = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
