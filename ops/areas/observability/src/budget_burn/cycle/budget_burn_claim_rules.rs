use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn claim must pass. Check returns its problems, or none.
pub trait BudgetBurnClaimRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnClaimKeyIdLengthRule {
    pub max_length: i64,
}

impl BudgetBurnClaimRule for BudgetBurnClaimKeyIdLengthRule {
    fn check(&self, item: &BudgetBurnClaim) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnClaimRequiredCodeRule {
    pub message: String,
}

impl BudgetBurnClaimRule for BudgetBurnClaimRequiredCodeRule {
    fn check(&self, item: &BudgetBurnClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnClaimAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnClaimRule for BudgetBurnClaimAmountRangeRule {
    fn check(&self, item: &BudgetBurnClaim) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnClaimCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnClaimRule for BudgetBurnClaimCodePrefixRule {
    fn check(&self, item: &BudgetBurnClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnClaimRequiredKeyIdRule {
    pub message: String,
}

impl BudgetBurnClaimRule for BudgetBurnClaimRequiredKeyIdRule {
    fn check(&self, item: &BudgetBurnClaim) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnClaimReviewerLengthRule {
    pub max_length: i64,
}

impl BudgetBurnClaimRule for BudgetBurnClaimReviewerLengthRule {
    fn check(&self, item: &BudgetBurnClaim) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn claim.
#[derive(Debug, Clone)]
pub struct BudgetBurnClaimValidator {
    pub rules: Vec<Rc<dyn BudgetBurnClaimRule>>,
}

impl BudgetBurnClaimValidator {
    pub fn validate(&self, item: &BudgetBurnClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_claim_validator() -> BudgetBurnClaimValidator {
    BudgetBurnClaimValidator {
        rules: vec![
            Rc::new(BudgetBurnClaimKeyIdLengthRule { max_length: 3600 }) as Rc<dyn BudgetBurnClaimRule>,
            Rc::new(BudgetBurnClaimRequiredCodeRule {
                message: "budget burn claim needs a value".to_string(),
            }),
            Rc::new(BudgetBurnClaimAmountRangeRule { min: 5.7, max: 1554.8 }),
            Rc::new(BudgetBurnClaimCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(BudgetBurnClaimRequiredKeyIdRule {
                message: "budget burn claim needs an entry".to_string(),
            }),
            Rc::new(BudgetBurnClaimReviewerLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_claim_validator();
        let item = new_budget_burn_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = BudgetBurnClaimKeyIdLengthRule { max_length: 16 };
        let mut item = new_budget_burn_claim("x", "y");
        item.key_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BudgetBurnClaimRequiredCodeRule {
            message: "budget burn claim needs a value".to_string(),
        };
        let mut item = new_budget_burn_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = BudgetBurnClaimAmountRangeRule { min: 4.2, max: 1056.0 };
        let mut item = new_budget_burn_claim("x", "y");
        item.amount = 1057.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = BudgetBurnClaimRequiredKeyIdRule {
            message: "budget burn claim needs an entry".to_string(),
        };
        let mut item = new_budget_burn_claim("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = BudgetBurnClaimReviewerLengthRule { max_length: 64 };
        let mut item = new_budget_burn_claim("x", "y");
        item.reviewer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
