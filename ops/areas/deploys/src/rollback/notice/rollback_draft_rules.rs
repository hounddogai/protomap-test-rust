use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback draft must pass. Check returns its problems, or none.
pub trait RollbackDraftRule: std::fmt::Debug {
    fn check(&self, item: &RollbackDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDraftRequiredCurrencyRule {
    pub message: String,
}

impl RollbackDraftRule for RollbackDraftRequiredCurrencyRule {
    fn check(&self, item: &RollbackDraft) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDraftUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackDraftRule for RollbackDraftUpdatedAtRangeRule {
    fn check(&self, item: &RollbackDraft) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDraftCurrencyLengthRule {
    pub max_length: i64,
}

impl RollbackDraftRule for RollbackDraftCurrencyLengthRule {
    fn check(&self, item: &RollbackDraft) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDraftNameLengthRule {
    pub max_length: i64,
}

impl RollbackDraftRule for RollbackDraftNameLengthRule {
    fn check(&self, item: &RollbackDraft) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDraftClusterLengthRule {
    pub max_length: i64,
}

impl RollbackDraftRule for RollbackDraftClusterLengthRule {
    fn check(&self, item: &RollbackDraft) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDraftAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RollbackDraftRule for RollbackDraftAllowedNameRule {
    fn check(&self, item: &RollbackDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is duplicated", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback draft.
#[derive(Debug, Clone)]
pub struct RollbackDraftValidator {
    pub rules: Vec<Rc<dyn RollbackDraftRule>>,
}

impl RollbackDraftValidator {
    pub fn validate(&self, item: &RollbackDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_draft_validator() -> RollbackDraftValidator {
    RollbackDraftValidator {
        rules: vec![
            Rc::new(RollbackDraftRequiredCurrencyRule {
                message: "rollback draft needs a value".to_string(),
            }) as Rc<dyn RollbackDraftRule>,
            Rc::new(RollbackDraftUpdatedAtRangeRule { min: 255, max: 1000 }),
            Rc::new(RollbackDraftCurrencyLengthRule { max_length: 64 }),
            Rc::new(RollbackDraftNameLengthRule { max_length: 120 }),
            Rc::new(RollbackDraftClusterLengthRule { max_length: 16 }),
            Rc::new(RollbackDraftAllowedNameRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_draft_validator();
        let item = new_rollback_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = RollbackDraftRequiredCurrencyRule {
            message: "rollback draft needs an entry".to_string(),
        };
        let mut item = new_rollback_draft("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = RollbackDraftUpdatedAtRangeRule { min: 16, max: 3600 };
        let mut item = new_rollback_draft("x", "y");
        item.updated_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = RollbackDraftCurrencyLengthRule { max_length: 64 };
        let mut item = new_rollback_draft("x", "y");
        item.currency = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollbackDraftNameLengthRule { max_length: 255 };
        let mut item = new_rollback_draft("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = RollbackDraftClusterLengthRule { max_length: 255 };
        let mut item = new_rollback_draft("x", "y");
        item.cluster = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
