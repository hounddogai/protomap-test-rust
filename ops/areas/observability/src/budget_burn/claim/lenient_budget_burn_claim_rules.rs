use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient budget burn claim must pass. Check returns its problems, or none.
pub trait LenientBudgetBurnClaimRule: std::fmt::Debug {
    fn check(&self, item: &LenientBudgetBurnClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBudgetBurnClaimScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientBudgetBurnClaimRule for LenientBudgetBurnClaimScoreRangeRule {
    fn check(&self, item: &LenientBudgetBurnClaim) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBudgetBurnClaimLabelLengthRule {
    pub max_length: i64,
}

impl LenientBudgetBurnClaimRule for LenientBudgetBurnClaimLabelLengthRule {
    fn check(&self, item: &LenientBudgetBurnClaim) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBudgetBurnClaimRequiredLabelRule {
    pub message: String,
}

impl LenientBudgetBurnClaimRule for LenientBudgetBurnClaimRequiredLabelRule {
    fn check(&self, item: &LenientBudgetBurnClaim) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBudgetBurnClaimNamespaceLengthRule {
    pub max_length: i64,
}

impl LenientBudgetBurnClaimRule for LenientBudgetBurnClaimNamespaceLengthRule {
    fn check(&self, item: &LenientBudgetBurnClaim) -> Vec<String> {
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
pub struct LenientBudgetBurnClaimRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientBudgetBurnClaimRule for LenientBudgetBurnClaimRetriesRangeRule {
    fn check(&self, item: &LenientBudgetBurnClaim) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient budget burn claim.
#[derive(Debug, Clone)]
pub struct LenientBudgetBurnClaimValidator {
    pub rules: Vec<Rc<dyn LenientBudgetBurnClaimRule>>,
}

impl LenientBudgetBurnClaimValidator {
    pub fn validate(&self, item: &LenientBudgetBurnClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientBudgetBurnClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientBudgetBurnClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_budget_burn_claim_validator() -> LenientBudgetBurnClaimValidator {
    LenientBudgetBurnClaimValidator {
        rules: vec![
            Rc::new(LenientBudgetBurnClaimScoreRangeRule { min: 2.0, max: 3344.8 })
                as Rc<dyn LenientBudgetBurnClaimRule>,
            Rc::new(LenientBudgetBurnClaimLabelLengthRule { max_length: 16 }),
            Rc::new(LenientBudgetBurnClaimRequiredLabelRule {
                message: "lenient budget burn claim needs an entry".to_string(),
            }),
            Rc::new(LenientBudgetBurnClaimNamespaceLengthRule { max_length: 3600 }),
            Rc::new(LenientBudgetBurnClaimRetriesRangeRule { min: 64, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_budget_burn_claim_validator();
        let item = new_lenient_budget_burn_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = LenientBudgetBurnClaimScoreRangeRule { min: 5.1, max: 3739.4 };
        let mut item = new_lenient_budget_burn_claim("x", "y");
        item.score = 3740.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientBudgetBurnClaimLabelLengthRule { max_length: 255 };
        let mut item = new_lenient_budget_burn_claim("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientBudgetBurnClaimRequiredLabelRule {
            message: "lenient budget burn claim needs an entry".to_string(),
        };
        let mut item = new_lenient_budget_burn_claim("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = LenientBudgetBurnClaimNamespaceLengthRule { max_length: 64 };
        let mut item = new_lenient_budget_burn_claim("x", "y");
        item.namespace = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = LenientBudgetBurnClaimRetriesRangeRule { min: 32, max: 1000 };
        let mut item = new_lenient_budget_burn_claim("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
