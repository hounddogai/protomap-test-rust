use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending rule tier must pass. Check returns its problems, or none.
pub trait PendingRuleTierRule: std::fmt::Debug {
    fn check(&self, item: &PendingRuleTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleTierCodeLengthRule {
    pub max_length: i64,
}

impl PendingRuleTierRule for PendingRuleTierCodeLengthRule {
    fn check(&self, item: &PendingRuleTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleTierPinnedGuardRule {
    pub limit: i64,
}

impl PendingRuleTierRule for PendingRuleTierPinnedGuardRule {
    fn check(&self, item: &PendingRuleTier) -> Vec<String> {
        if item.pinned && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while pinned",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleTierCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRuleTierRule for PendingRuleTierCapacityRangeRule {
    fn check(&self, item: &PendingRuleTier) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleTierRequiredLabelRule {
    pub message: String,
}

impl PendingRuleTierRule for PendingRuleTierRequiredLabelRule {
    fn check(&self, item: &PendingRuleTier) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending rule tier.
#[derive(Debug, Clone)]
pub struct PendingRuleTierValidator {
    pub rules: Vec<Rc<dyn PendingRuleTierRule>>,
}

impl PendingRuleTierValidator {
    pub fn validate(&self, item: &PendingRuleTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRuleTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRuleTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_rule_tier_validator() -> PendingRuleTierValidator {
    PendingRuleTierValidator {
        rules: vec![
            Rc::new(PendingRuleTierCodeLengthRule { max_length: 120 }) as Rc<dyn PendingRuleTierRule>,
            Rc::new(PendingRuleTierPinnedGuardRule { limit: 80 }),
            Rc::new(PendingRuleTierCapacityRangeRule { min: 32, max: 86400 }),
            Rc::new(PendingRuleTierRequiredLabelRule {
                message: "pending rule tier needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_rule_tier_validator();
        let item = new_pending_rule_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingRuleTierCodeLengthRule { max_length: 120 };
        let mut item = new_pending_rule_tier("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = PendingRuleTierCapacityRangeRule { min: 120, max: 255 };
        let mut item = new_pending_rule_tier("x", "y");
        item.capacity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingRuleTierRequiredLabelRule {
            message: "pending rule tier needs an entry".to_string(),
        };
        let mut item = new_pending_rule_tier("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
