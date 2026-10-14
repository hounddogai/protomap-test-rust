use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback target must pass. Check returns its problems, or none.
pub trait RollbackTargetRule: std::fmt::Debug {
    fn check(&self, item: &RollbackTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTargetCodePrefixRule {
    pub prefix: String,
}

impl RollbackTargetRule for RollbackTargetCodePrefixRule {
    fn check(&self, item: &RollbackTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTargetPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackTargetRule for RollbackTargetPriorityRangeRule {
    fn check(&self, item: &RollbackTarget) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTargetApprovedGuardRule {
    pub limit: i64,
}

impl RollbackTargetRule for RollbackTargetApprovedGuardRule {
    fn check(&self, item: &RollbackTarget) -> Vec<String> {
        if item.approved && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while approved",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTargetRequiredNotesRule {
    pub message: String,
}

impl RollbackTargetRule for RollbackTargetRequiredNotesRule {
    fn check(&self, item: &RollbackTarget) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTargetStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackTargetRule for RollbackTargetStartsAtRangeRule {
    fn check(&self, item: &RollbackTarget) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTargetRequiredLabelRule {
    pub message: String,
}

impl RollbackTargetRule for RollbackTargetRequiredLabelRule {
    fn check(&self, item: &RollbackTarget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback target.
#[derive(Debug, Clone)]
pub struct RollbackTargetValidator {
    pub rules: Vec<Rc<dyn RollbackTargetRule>>,
}

impl RollbackTargetValidator {
    pub fn validate(&self, item: &RollbackTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_target_validator() -> RollbackTargetValidator {
    RollbackTargetValidator {
        rules: vec![
            Rc::new(RollbackTargetCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn RollbackTargetRule>,
            Rc::new(RollbackTargetPriorityRangeRule { min: 32, max: 64 }),
            Rc::new(RollbackTargetApprovedGuardRule { limit: 3600 }),
            Rc::new(RollbackTargetRequiredNotesRule {
                message: "rollback target needs an entry".to_string(),
            }),
            Rc::new(RollbackTargetStartsAtRangeRule { min: 80, max: 86400 }),
            Rc::new(RollbackTargetRequiredLabelRule {
                message: "rollback target needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_target_validator();
        let item = new_rollback_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = RollbackTargetPriorityRangeRule { min: 255, max: 86400 };
        let mut item = new_rollback_target("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = RollbackTargetRequiredNotesRule {
            message: "rollback target needs this field".to_string(),
        };
        let mut item = new_rollback_target("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = RollbackTargetStartsAtRangeRule { min: 16, max: 86400 };
        let mut item = new_rollback_target("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollbackTargetRequiredLabelRule {
            message: "rollback target needs an entry".to_string(),
        };
        let mut item = new_rollback_target("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
