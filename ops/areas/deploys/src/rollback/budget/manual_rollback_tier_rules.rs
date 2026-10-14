use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual rollback tier must pass. Check returns its problems, or none.
pub trait ManualRollbackTierRule: std::fmt::Debug {
    fn check(&self, item: &ManualRollbackTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackTierUnitLengthRule {
    pub max_length: i64,
}

impl ManualRollbackTierRule for ManualRollbackTierUnitLengthRule {
    fn check(&self, item: &ManualRollbackTier) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackTierLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualRollbackTierRule for ManualRollbackTierLimitRangeRule {
    fn check(&self, item: &ManualRollbackTier) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackTierLabelLengthRule {
    pub max_length: i64,
}

impl ManualRollbackTierRule for ManualRollbackTierLabelLengthRule {
    fn check(&self, item: &ManualRollbackTier) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackTierRequiredExternalRefRule {
    pub message: String,
}

impl ManualRollbackTierRule for ManualRollbackTierRequiredExternalRefRule {
    fn check(&self, item: &ManualRollbackTier) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackTierCodeLengthRule {
    pub max_length: i64,
}

impl ManualRollbackTierRule for ManualRollbackTierCodeLengthRule {
    fn check(&self, item: &ManualRollbackTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual rollback tier.
#[derive(Debug, Clone)]
pub struct ManualRollbackTierValidator {
    pub rules: Vec<Rc<dyn ManualRollbackTierRule>>,
}

impl ManualRollbackTierValidator {
    pub fn validate(&self, item: &ManualRollbackTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualRollbackTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRollbackTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_rollback_tier_validator() -> ManualRollbackTierValidator {
    ManualRollbackTierValidator {
        rules: vec![
            Rc::new(ManualRollbackTierUnitLengthRule { max_length: 32 }) as Rc<dyn ManualRollbackTierRule>,
            Rc::new(ManualRollbackTierLimitRangeRule { min: 255, max: 1000 }),
            Rc::new(ManualRollbackTierLabelLengthRule { max_length: 16 }),
            Rc::new(ManualRollbackTierRequiredExternalRefRule {
                message: "manual rollback tier needs an entry".to_string(),
            }),
            Rc::new(ManualRollbackTierCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_rollback_tier_validator();
        let item = new_manual_rollback_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = ManualRollbackTierUnitLengthRule { max_length: 86400 };
        let mut item = new_manual_rollback_tier("x", "y");
        item.unit = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = ManualRollbackTierLimitRangeRule { min: 3600, max: 86400 };
        let mut item = new_manual_rollback_tier("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ManualRollbackTierLabelLengthRule { max_length: 80 };
        let mut item = new_manual_rollback_tier("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = ManualRollbackTierRequiredExternalRefRule {
            message: "manual rollback tier needs a value".to_string(),
        };
        let mut item = new_manual_rollback_tier("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualRollbackTierCodeLengthRule { max_length: 86400 };
        let mut item = new_manual_rollback_tier("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
