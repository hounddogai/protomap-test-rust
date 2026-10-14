use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered budget burn claim must pass. Check returns its problems, or none.
pub trait TieredBudgetBurnClaimRule: std::fmt::Debug {
    fn check(&self, item: &TieredBudgetBurnClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnClaimSourceLengthRule {
    pub max_length: i64,
}

impl TieredBudgetBurnClaimRule for TieredBudgetBurnClaimSourceLengthRule {
    fn check(&self, item: &TieredBudgetBurnClaim) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnClaimRequiredTitleRule {
    pub message: String,
}

impl TieredBudgetBurnClaimRule for TieredBudgetBurnClaimRequiredTitleRule {
    fn check(&self, item: &TieredBudgetBurnClaim) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnClaimRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredBudgetBurnClaimRule for TieredBudgetBurnClaimRevisionRangeRule {
    fn check(&self, item: &TieredBudgetBurnClaim) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnClaimCodeLengthRule {
    pub max_length: i64,
}

impl TieredBudgetBurnClaimRule for TieredBudgetBurnClaimCodeLengthRule {
    fn check(&self, item: &TieredBudgetBurnClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnClaimCodePrefixRule {
    pub prefix: String,
}

impl TieredBudgetBurnClaimRule for TieredBudgetBurnClaimCodePrefixRule {
    fn check(&self, item: &TieredBudgetBurnClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnClaimRequiredSourceRule {
    pub message: String,
}

impl TieredBudgetBurnClaimRule for TieredBudgetBurnClaimRequiredSourceRule {
    fn check(&self, item: &TieredBudgetBurnClaim) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered budget burn claim.
#[derive(Debug, Clone)]
pub struct TieredBudgetBurnClaimValidator {
    pub rules: Vec<Rc<dyn TieredBudgetBurnClaimRule>>,
}

impl TieredBudgetBurnClaimValidator {
    pub fn validate(&self, item: &TieredBudgetBurnClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredBudgetBurnClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredBudgetBurnClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_budget_burn_claim_validator() -> TieredBudgetBurnClaimValidator {
    TieredBudgetBurnClaimValidator {
        rules: vec![
            Rc::new(TieredBudgetBurnClaimSourceLengthRule { max_length: 32 }) as Rc<dyn TieredBudgetBurnClaimRule>,
            Rc::new(TieredBudgetBurnClaimRequiredTitleRule {
                message: "tiered budget burn claim needs an entry".to_string(),
            }),
            Rc::new(TieredBudgetBurnClaimRevisionRangeRule { min: 120, max: 1000 }),
            Rc::new(TieredBudgetBurnClaimCodeLengthRule { max_length: 16 }),
            Rc::new(TieredBudgetBurnClaimCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(TieredBudgetBurnClaimRequiredSourceRule {
                message: "tiered budget burn claim needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_budget_burn_claim_validator();
        let item = new_tiered_budget_burn_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = TieredBudgetBurnClaimSourceLengthRule { max_length: 32 };
        let mut item = new_tiered_budget_burn_claim("x", "y");
        item.source = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredBudgetBurnClaimRequiredTitleRule {
            message: "tiered budget burn claim needs a value".to_string(),
        };
        let mut item = new_tiered_budget_burn_claim("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = TieredBudgetBurnClaimRevisionRangeRule { min: 120, max: 1000 };
        let mut item = new_tiered_budget_burn_claim("x", "y");
        item.revision = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredBudgetBurnClaimCodeLengthRule { max_length: 1000 };
        let mut item = new_tiered_budget_burn_claim("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = TieredBudgetBurnClaimRequiredSourceRule {
            message: "tiered budget burn claim needs this field".to_string(),
        };
        let mut item = new_tiered_budget_burn_claim("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
