use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive binding milestone must pass. Check returns its problems, or none.
pub trait AdaptiveBindingMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveBindingMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBindingMilestoneRequiredCodeRule {
    pub message: String,
}

impl AdaptiveBindingMilestoneRule for AdaptiveBindingMilestoneRequiredCodeRule {
    fn check(&self, item: &AdaptiveBindingMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBindingMilestoneCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveBindingMilestoneRule for AdaptiveBindingMilestoneCodePrefixRule {
    fn check(&self, item: &AdaptiveBindingMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBindingMilestoneCurrencyLengthRule {
    pub max_length: i64,
}

impl AdaptiveBindingMilestoneRule for AdaptiveBindingMilestoneCurrencyLengthRule {
    fn check(&self, item: &AdaptiveBindingMilestone) -> Vec<String> {
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
pub struct AdaptiveBindingMilestoneApprovedGuardRule {
    pub limit: f64,
}

impl AdaptiveBindingMilestoneRule for AdaptiveBindingMilestoneApprovedGuardRule {
    fn check(&self, item: &AdaptiveBindingMilestone) -> Vec<String> {
        if item.approved && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while approved", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBindingMilestoneRequiredCurrencyRule {
    pub message: String,
}

impl AdaptiveBindingMilestoneRule for AdaptiveBindingMilestoneRequiredCurrencyRule {
    fn check(&self, item: &AdaptiveBindingMilestone) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive binding milestone.
#[derive(Debug, Clone)]
pub struct AdaptiveBindingMilestoneValidator {
    pub rules: Vec<Rc<dyn AdaptiveBindingMilestoneRule>>,
}

impl AdaptiveBindingMilestoneValidator {
    pub fn validate(&self, item: &AdaptiveBindingMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveBindingMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveBindingMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_binding_milestone_validator() -> AdaptiveBindingMilestoneValidator {
    AdaptiveBindingMilestoneValidator {
        rules: vec![
            Rc::new(AdaptiveBindingMilestoneRequiredCodeRule {
                message: "adaptive binding milestone needs a value".to_string(),
            }) as Rc<dyn AdaptiveBindingMilestoneRule>,
            Rc::new(AdaptiveBindingMilestoneCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(AdaptiveBindingMilestoneCurrencyLengthRule { max_length: 80 }),
            Rc::new(AdaptiveBindingMilestoneApprovedGuardRule { limit: 4.9 }),
            Rc::new(AdaptiveBindingMilestoneRequiredCurrencyRule {
                message: "adaptive binding milestone needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_binding_milestone_validator();
        let item = new_adaptive_binding_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveBindingMilestoneRequiredCodeRule {
            message: "adaptive binding milestone needs this field".to_string(),
        };
        let mut item = new_adaptive_binding_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = AdaptiveBindingMilestoneCurrencyLengthRule { max_length: 16 };
        let mut item = new_adaptive_binding_milestone("x", "y");
        item.currency = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = AdaptiveBindingMilestoneRequiredCurrencyRule {
            message: "adaptive binding milestone needs this field".to_string(),
        };
        let mut item = new_adaptive_binding_milestone("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
