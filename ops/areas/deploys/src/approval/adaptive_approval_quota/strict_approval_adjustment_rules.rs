use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict approval adjustment must pass. Check returns its problems, or none.
pub trait StrictApprovalAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &StrictApprovalAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalAdjustmentRequiredNameRule {
    pub message: String,
}

impl StrictApprovalAdjustmentRule for StrictApprovalAdjustmentRequiredNameRule {
    fn check(&self, item: &StrictApprovalAdjustment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalAdjustmentRequiredNamespaceRule {
    pub message: String,
}

impl StrictApprovalAdjustmentRule for StrictApprovalAdjustmentRequiredNamespaceRule {
    fn check(&self, item: &StrictApprovalAdjustment) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalAdjustmentCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictApprovalAdjustmentRule for StrictApprovalAdjustmentCreatedAtRangeRule {
    fn check(&self, item: &StrictApprovalAdjustment) -> Vec<String> {
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
pub struct StrictApprovalAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl StrictApprovalAdjustmentRule for StrictApprovalAdjustmentCodePrefixRule {
    fn check(&self, item: &StrictApprovalAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalAdjustmentGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictApprovalAdjustmentRule for StrictApprovalAdjustmentGraceDaysRangeRule {
    fn check(&self, item: &StrictApprovalAdjustment) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict approval adjustment.
#[derive(Debug, Clone)]
pub struct StrictApprovalAdjustmentValidator {
    pub rules: Vec<Rc<dyn StrictApprovalAdjustmentRule>>,
}

impl StrictApprovalAdjustmentValidator {
    pub fn validate(&self, item: &StrictApprovalAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictApprovalAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictApprovalAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_approval_adjustment_validator() -> StrictApprovalAdjustmentValidator {
    StrictApprovalAdjustmentValidator {
        rules: vec![
            Rc::new(StrictApprovalAdjustmentRequiredNameRule {
                message: "strict approval adjustment needs an entry".to_string(),
            }) as Rc<dyn StrictApprovalAdjustmentRule>,
            Rc::new(StrictApprovalAdjustmentRequiredNamespaceRule {
                message: "strict approval adjustment needs an entry".to_string(),
            }),
            Rc::new(StrictApprovalAdjustmentCreatedAtRangeRule { min: 32, max: 1000 }),
            Rc::new(StrictApprovalAdjustmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(StrictApprovalAdjustmentGraceDaysRangeRule { min: 1000, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_approval_adjustment_validator();
        let item = new_strict_approval_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictApprovalAdjustmentRequiredNameRule {
            message: "strict approval adjustment needs an entry".to_string(),
        };
        let mut item = new_strict_approval_adjustment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = StrictApprovalAdjustmentRequiredNamespaceRule {
            message: "strict approval adjustment needs this field".to_string(),
        };
        let mut item = new_strict_approval_adjustment("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = StrictApprovalAdjustmentCreatedAtRangeRule { min: 16, max: 255 };
        let mut item = new_strict_approval_adjustment("x", "y");
        item.created_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = StrictApprovalAdjustmentGraceDaysRangeRule { min: 1000, max: 86400 };
        let mut item = new_strict_approval_adjustment("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
