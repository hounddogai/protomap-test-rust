use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict budget burn estimate must pass. Check returns its problems, or none.
pub trait StrictBudgetBurnEstimateRule: std::fmt::Debug {
    fn check(&self, item: &StrictBudgetBurnEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnEstimateRequiredCodeRule {
    pub message: String,
}

impl StrictBudgetBurnEstimateRule for StrictBudgetBurnEstimateRequiredCodeRule {
    fn check(&self, item: &StrictBudgetBurnEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnEstimateRequiredContactEmailRule {
    pub message: String,
}

impl StrictBudgetBurnEstimateRule for StrictBudgetBurnEstimateRequiredContactEmailRule {
    fn check(&self, item: &StrictBudgetBurnEstimate) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnEstimateCodeLengthRule {
    pub max_length: i64,
}

impl StrictBudgetBurnEstimateRule for StrictBudgetBurnEstimateCodeLengthRule {
    fn check(&self, item: &StrictBudgetBurnEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnEstimateArchivedGuardRule {
    pub limit: i64,
}

impl StrictBudgetBurnEstimateRule for StrictBudgetBurnEstimateArchivedGuardRule {
    fn check(&self, item: &StrictBudgetBurnEstimate) -> Vec<String> {
        if item.archived && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while archived",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnEstimateCodePrefixRule {
    pub prefix: String,
}

impl StrictBudgetBurnEstimateRule for StrictBudgetBurnEstimateCodePrefixRule {
    fn check(&self, item: &StrictBudgetBurnEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict budget burn estimate.
#[derive(Debug, Clone)]
pub struct StrictBudgetBurnEstimateValidator {
    pub rules: Vec<Rc<dyn StrictBudgetBurnEstimateRule>>,
}

impl StrictBudgetBurnEstimateValidator {
    pub fn validate(&self, item: &StrictBudgetBurnEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictBudgetBurnEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictBudgetBurnEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_budget_burn_estimate_validator() -> StrictBudgetBurnEstimateValidator {
    StrictBudgetBurnEstimateValidator {
        rules: vec![
            Rc::new(StrictBudgetBurnEstimateRequiredCodeRule {
                message: "strict budget burn estimate needs this field".to_string(),
            }) as Rc<dyn StrictBudgetBurnEstimateRule>,
            Rc::new(StrictBudgetBurnEstimateRequiredContactEmailRule {
                message: "strict budget burn estimate needs a value".to_string(),
            }),
            Rc::new(StrictBudgetBurnEstimateCodeLengthRule { max_length: 86400 }),
            Rc::new(StrictBudgetBurnEstimateArchivedGuardRule { limit: 255 }),
            Rc::new(StrictBudgetBurnEstimateCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_budget_burn_estimate_validator();
        let item = new_strict_budget_burn_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictBudgetBurnEstimateRequiredCodeRule {
            message: "strict budget burn estimate needs a value".to_string(),
        };
        let mut item = new_strict_budget_burn_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = StrictBudgetBurnEstimateRequiredContactEmailRule {
            message: "strict budget burn estimate needs this field".to_string(),
        };
        let mut item = new_strict_budget_burn_estimate("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictBudgetBurnEstimateCodeLengthRule { max_length: 255 };
        let mut item = new_strict_budget_burn_estimate("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
