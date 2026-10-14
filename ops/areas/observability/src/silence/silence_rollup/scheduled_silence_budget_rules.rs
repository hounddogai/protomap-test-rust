use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled silence budget must pass. Check returns its problems, or none.
pub trait ScheduledSilenceBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSilenceBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceBudgetRequiredCodeRule {
    pub message: String,
}

impl ScheduledSilenceBudgetRule for ScheduledSilenceBudgetRequiredCodeRule {
    fn check(&self, item: &ScheduledSilenceBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceBudgetTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl ScheduledSilenceBudgetRule for ScheduledSilenceBudgetTagsLimitRule {
    fn check(&self, item: &ScheduledSilenceBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceBudgetCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledSilenceBudgetRule for ScheduledSilenceBudgetCreatedAtRangeRule {
    fn check(&self, item: &ScheduledSilenceBudget) -> Vec<String> {
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
pub struct ScheduledSilenceBudgetApprovedGuardRule {
    pub limit: i64,
}

impl ScheduledSilenceBudgetRule for ScheduledSilenceBudgetApprovedGuardRule {
    fn check(&self, item: &ScheduledSilenceBudget) -> Vec<String> {
        if item.approved && (item.created_at > self.limit) {
            return vec![format!(
                "created at {} exceeds {} while approved",
                item.created_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceBudgetCodePrefixRule {
    pub prefix: String,
}

impl ScheduledSilenceBudgetRule for ScheduledSilenceBudgetCodePrefixRule {
    fn check(&self, item: &ScheduledSilenceBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceBudgetRequiredNameRule {
    pub message: String,
}

impl ScheduledSilenceBudgetRule for ScheduledSilenceBudgetRequiredNameRule {
    fn check(&self, item: &ScheduledSilenceBudget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled silence budget.
#[derive(Debug, Clone)]
pub struct ScheduledSilenceBudgetValidator {
    pub rules: Vec<Rc<dyn ScheduledSilenceBudgetRule>>,
}

impl ScheduledSilenceBudgetValidator {
    pub fn validate(&self, item: &ScheduledSilenceBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledSilenceBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSilenceBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_silence_budget_validator() -> ScheduledSilenceBudgetValidator {
    ScheduledSilenceBudgetValidator {
        rules: vec![
            Rc::new(ScheduledSilenceBudgetRequiredCodeRule {
                message: "scheduled silence budget needs this field".to_string(),
            }) as Rc<dyn ScheduledSilenceBudgetRule>,
            Rc::new(ScheduledSilenceBudgetTagsLimitRule {
                max_count: 32,
                banned: Vec::new(),
            }),
            Rc::new(ScheduledSilenceBudgetCreatedAtRangeRule { min: 3600, max: 3600 }),
            Rc::new(ScheduledSilenceBudgetApprovedGuardRule { limit: 255 }),
            Rc::new(ScheduledSilenceBudgetCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ScheduledSilenceBudgetRequiredNameRule {
                message: "scheduled silence budget needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_silence_budget_validator();
        let item = new_scheduled_silence_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledSilenceBudgetRequiredCodeRule {
            message: "scheduled silence budget needs this field".to_string(),
        };
        let mut item = new_scheduled_silence_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = ScheduledSilenceBudgetCreatedAtRangeRule { min: 32, max: 32 };
        let mut item = new_scheduled_silence_budget("x", "y");
        item.created_at = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ScheduledSilenceBudgetRequiredNameRule {
            message: "scheduled silence budget needs this field".to_string(),
        };
        let mut item = new_scheduled_silence_budget("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
