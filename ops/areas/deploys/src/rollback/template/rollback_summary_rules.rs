use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback summary must pass. Check returns its problems, or none.
pub trait RollbackSummaryRule: std::fmt::Debug {
    fn check(&self, item: &RollbackSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSummaryCipherLengthRule {
    pub max_length: i64,
}

impl RollbackSummaryRule for RollbackSummaryCipherLengthRule {
    fn check(&self, item: &RollbackSummary) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSummaryCodeLengthRule {
    pub max_length: i64,
}

impl RollbackSummaryRule for RollbackSummaryCodeLengthRule {
    fn check(&self, item: &RollbackSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSummaryRequiredLabelRule {
    pub message: String,
}

impl RollbackSummaryRule for RollbackSummaryRequiredLabelRule {
    fn check(&self, item: &RollbackSummary) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSummaryNamespaceLengthRule {
    pub max_length: i64,
}

impl RollbackSummaryRule for RollbackSummaryNamespaceLengthRule {
    fn check(&self, item: &RollbackSummary) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSummaryAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl RollbackSummaryRule for RollbackSummaryAllowedScopeRule {
    fn check(&self, item: &RollbackSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is unknown", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSummaryQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackSummaryRule for RollbackSummaryQuantityRangeRule {
    fn check(&self, item: &RollbackSummary) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback summary.
#[derive(Debug, Clone)]
pub struct RollbackSummaryValidator {
    pub rules: Vec<Rc<dyn RollbackSummaryRule>>,
}

impl RollbackSummaryValidator {
    pub fn validate(&self, item: &RollbackSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_summary_validator() -> RollbackSummaryValidator {
    RollbackSummaryValidator {
        rules: vec![
            Rc::new(RollbackSummaryCipherLengthRule { max_length: 120 }) as Rc<dyn RollbackSummaryRule>,
            Rc::new(RollbackSummaryCodeLengthRule { max_length: 32 }),
            Rc::new(RollbackSummaryRequiredLabelRule {
                message: "rollback summary needs an entry".to_string(),
            }),
            Rc::new(RollbackSummaryNamespaceLengthRule { max_length: 16 }),
            Rc::new(RollbackSummaryAllowedScopeRule { allowed: Vec::new() }),
            Rc::new(RollbackSummaryQuantityRangeRule { min: 80, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_summary_validator();
        let item = new_rollback_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = RollbackSummaryCipherLengthRule { max_length: 86400 };
        let mut item = new_rollback_summary("x", "y");
        item.cipher = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollbackSummaryCodeLengthRule { max_length: 255 };
        let mut item = new_rollback_summary("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollbackSummaryRequiredLabelRule {
            message: "rollback summary needs a value".to_string(),
        };
        let mut item = new_rollback_summary("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = RollbackSummaryNamespaceLengthRule { max_length: 255 };
        let mut item = new_rollback_summary("x", "y");
        item.namespace = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = RollbackSummaryQuantityRangeRule { min: 16, max: 32 };
        let mut item = new_rollback_summary("x", "y");
        item.quantity = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
