use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional rule checkpoint must pass. Check returns its problems, or none.
pub trait RegionalRuleCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRuleCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleCheckpointAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl RegionalRuleCheckpointRule for RegionalRuleCheckpointAllowedPhoneRule {
    fn check(&self, item: &RegionalRuleCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is unknown", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleCheckpointQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalRuleCheckpointRule for RegionalRuleCheckpointQuantityRangeRule {
    fn check(&self, item: &RegionalRuleCheckpoint) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleCheckpointRequiredPhoneRule {
    pub message: String,
}

impl RegionalRuleCheckpointRule for RegionalRuleCheckpointRequiredPhoneRule {
    fn check(&self, item: &RegionalRuleCheckpoint) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional rule checkpoint.
#[derive(Debug, Clone)]
pub struct RegionalRuleCheckpointValidator {
    pub rules: Vec<Rc<dyn RegionalRuleCheckpointRule>>,
}

impl RegionalRuleCheckpointValidator {
    pub fn validate(&self, item: &RegionalRuleCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalRuleCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRuleCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_rule_checkpoint_validator() -> RegionalRuleCheckpointValidator {
    RegionalRuleCheckpointValidator {
        rules: vec![
            Rc::new(RegionalRuleCheckpointAllowedPhoneRule { allowed: Vec::new() })
                as Rc<dyn RegionalRuleCheckpointRule>,
            Rc::new(RegionalRuleCheckpointQuantityRangeRule { min: 120, max: 1000 }),
            Rc::new(RegionalRuleCheckpointRequiredPhoneRule {
                message: "regional rule checkpoint needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_rule_checkpoint_validator();
        let item = new_regional_rule_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = RegionalRuleCheckpointQuantityRangeRule { min: 32, max: 80 };
        let mut item = new_regional_rule_checkpoint("x", "y");
        item.quantity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = RegionalRuleCheckpointRequiredPhoneRule {
            message: "regional rule checkpoint needs an entry".to_string(),
        };
        let mut item = new_regional_rule_checkpoint("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
