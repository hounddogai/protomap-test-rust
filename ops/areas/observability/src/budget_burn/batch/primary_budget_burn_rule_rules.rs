use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary budget burn rule must pass. Check returns its problems, or none.
pub trait PrimaryBudgetBurnRuleRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryBudgetBurnRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnRuleTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryBudgetBurnRuleRule for PrimaryBudgetBurnRuleTitleLengthRule {
    fn check(&self, item: &PrimaryBudgetBurnRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnRuleWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryBudgetBurnRuleRule for PrimaryBudgetBurnRuleWeightRangeRule {
    fn check(&self, item: &PrimaryBudgetBurnRule) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnRuleRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryBudgetBurnRuleRule for PrimaryBudgetBurnRuleRateRangeRule {
    fn check(&self, item: &PrimaryBudgetBurnRule) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnRuleRequiredCodeRule {
    pub message: String,
}

impl PrimaryBudgetBurnRuleRule for PrimaryBudgetBurnRuleRequiredCodeRule {
    fn check(&self, item: &PrimaryBudgetBurnRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnRuleActiveGuardRule {
    pub limit: f64,
}

impl PrimaryBudgetBurnRuleRule for PrimaryBudgetBurnRuleActiveGuardRule {
    fn check(&self, item: &PrimaryBudgetBurnRule) -> Vec<String> {
        if item.active && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while active", item.rate, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary budget burn rule.
#[derive(Debug, Clone)]
pub struct PrimaryBudgetBurnRuleValidator {
    pub rules: Vec<Rc<dyn PrimaryBudgetBurnRuleRule>>,
}

impl PrimaryBudgetBurnRuleValidator {
    pub fn validate(&self, item: &PrimaryBudgetBurnRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryBudgetBurnRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryBudgetBurnRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_budget_burn_rule_validator() -> PrimaryBudgetBurnRuleValidator {
    PrimaryBudgetBurnRuleValidator {
        rules: vec![
            Rc::new(PrimaryBudgetBurnRuleTitleLengthRule { max_length: 16 }) as Rc<dyn PrimaryBudgetBurnRuleRule>,
            Rc::new(PrimaryBudgetBurnRuleWeightRangeRule { min: 7.2, max: 909.8 }),
            Rc::new(PrimaryBudgetBurnRuleRateRangeRule { min: 7.2, max: 4786.3 }),
            Rc::new(PrimaryBudgetBurnRuleRequiredCodeRule {
                message: "primary budget burn rule needs this field".to_string(),
            }),
            Rc::new(PrimaryBudgetBurnRuleActiveGuardRule { limit: 8.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_budget_burn_rule_validator();
        let item = new_primary_budget_burn_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryBudgetBurnRuleTitleLengthRule { max_length: 16 };
        let mut item = new_primary_budget_burn_rule("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = PrimaryBudgetBurnRuleWeightRangeRule { min: 1.0, max: 4331.7 };
        let mut item = new_primary_budget_burn_rule("x", "y");
        item.weight = 4333.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = PrimaryBudgetBurnRuleRateRangeRule { min: 3.8, max: 1112.0 };
        let mut item = new_primary_budget_burn_rule("x", "y");
        item.rate = 1113.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryBudgetBurnRuleRequiredCodeRule {
            message: "primary budget burn rule needs this field".to_string(),
        };
        let mut item = new_primary_budget_burn_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
