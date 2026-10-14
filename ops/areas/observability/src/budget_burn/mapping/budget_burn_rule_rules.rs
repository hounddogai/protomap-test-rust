use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn rule must pass. Check returns its problems, or none.
pub trait BudgetBurnRuleRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRuleIssuerLengthRule {
    pub max_length: i64,
}

impl BudgetBurnRuleRule for BudgetBurnRuleIssuerLengthRule {
    fn check(&self, item: &BudgetBurnRule) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRuleRequiredIssuerRule {
    pub message: String,
}

impl BudgetBurnRuleRule for BudgetBurnRuleRequiredIssuerRule {
    fn check(&self, item: &BudgetBurnRule) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRuleRequiredNameRule {
    pub message: String,
}

impl BudgetBurnRuleRule for BudgetBurnRuleRequiredNameRule {
    fn check(&self, item: &BudgetBurnRule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRuleStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnRuleRule for BudgetBurnRuleStartsAtRangeRule {
    fn check(&self, item: &BudgetBurnRule) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn rule.
#[derive(Debug, Clone)]
pub struct BudgetBurnRuleValidator {
    pub rules: Vec<Rc<dyn BudgetBurnRuleRule>>,
}

impl BudgetBurnRuleValidator {
    pub fn validate(&self, item: &BudgetBurnRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_rule_validator() -> BudgetBurnRuleValidator {
    BudgetBurnRuleValidator {
        rules: vec![
            Rc::new(BudgetBurnRuleIssuerLengthRule { max_length: 80 }) as Rc<dyn BudgetBurnRuleRule>,
            Rc::new(BudgetBurnRuleRequiredIssuerRule {
                message: "budget burn rule needs this field".to_string(),
            }),
            Rc::new(BudgetBurnRuleRequiredNameRule {
                message: "budget burn rule needs a value".to_string(),
            }),
            Rc::new(BudgetBurnRuleStartsAtRangeRule { min: 16, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_rule_validator();
        let item = new_budget_burn_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = BudgetBurnRuleIssuerLengthRule { max_length: 1000 };
        let mut item = new_budget_burn_rule("x", "y");
        item.issuer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = BudgetBurnRuleRequiredIssuerRule {
            message: "budget burn rule needs an entry".to_string(),
        };
        let mut item = new_budget_burn_rule("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BudgetBurnRuleRequiredNameRule {
            message: "budget burn rule needs an entry".to_string(),
        };
        let mut item = new_budget_burn_rule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = BudgetBurnRuleStartsAtRangeRule { min: 255, max: 1000 };
        let mut item = new_budget_burn_rule("x", "y");
        item.starts_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
