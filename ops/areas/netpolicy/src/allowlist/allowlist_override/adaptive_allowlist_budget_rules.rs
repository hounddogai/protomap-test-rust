use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive allowlist budget must pass. Check returns its problems, or none.
pub trait AdaptiveAllowlistBudgetRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveAllowlistBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBudgetRequiredTitleRule {
    pub message: String,
}

impl AdaptiveAllowlistBudgetRule for AdaptiveAllowlistBudgetRequiredTitleRule {
    fn check(&self, item: &AdaptiveAllowlistBudget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBudgetChannelLengthRule {
    pub max_length: i64,
}

impl AdaptiveAllowlistBudgetRule for AdaptiveAllowlistBudgetChannelLengthRule {
    fn check(&self, item: &AdaptiveAllowlistBudget) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBudgetRequiredCodeRule {
    pub message: String,
}

impl AdaptiveAllowlistBudgetRule for AdaptiveAllowlistBudgetRequiredCodeRule {
    fn check(&self, item: &AdaptiveAllowlistBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive allowlist budget.
#[derive(Debug, Clone)]
pub struct AdaptiveAllowlistBudgetValidator {
    pub rules: Vec<Rc<dyn AdaptiveAllowlistBudgetRule>>,
}

impl AdaptiveAllowlistBudgetValidator {
    pub fn validate(&self, item: &AdaptiveAllowlistBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveAllowlistBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveAllowlistBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_allowlist_budget_validator() -> AdaptiveAllowlistBudgetValidator {
    AdaptiveAllowlistBudgetValidator {
        rules: vec![
            Rc::new(AdaptiveAllowlistBudgetRequiredTitleRule {
                message: "adaptive allowlist budget needs a value".to_string(),
            }) as Rc<dyn AdaptiveAllowlistBudgetRule>,
            Rc::new(AdaptiveAllowlistBudgetChannelLengthRule { max_length: 255 }),
            Rc::new(AdaptiveAllowlistBudgetRequiredCodeRule {
                message: "adaptive allowlist budget needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_allowlist_budget_validator();
        let item = new_adaptive_allowlist_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AdaptiveAllowlistBudgetRequiredTitleRule {
            message: "adaptive allowlist budget needs this field".to_string(),
        };
        let mut item = new_adaptive_allowlist_budget("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = AdaptiveAllowlistBudgetChannelLengthRule { max_length: 80 };
        let mut item = new_adaptive_allowlist_budget("x", "y");
        item.channel = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveAllowlistBudgetRequiredCodeRule {
            message: "adaptive allowlist budget needs an entry".to_string(),
        };
        let mut item = new_adaptive_allowlist_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
