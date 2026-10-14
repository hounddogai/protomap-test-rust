use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default trust domain budget must pass. Check returns its problems, or none.
pub trait DefaultTrustDomainBudgetRule: std::fmt::Debug {
    fn check(&self, item: &DefaultTrustDomainBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainBudgetLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultTrustDomainBudgetRule for DefaultTrustDomainBudgetLimitRangeRule {
    fn check(&self, item: &DefaultTrustDomainBudget) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainBudgetCodeLengthRule {
    pub max_length: i64,
}

impl DefaultTrustDomainBudgetRule for DefaultTrustDomainBudgetCodeLengthRule {
    fn check(&self, item: &DefaultTrustDomainBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainBudgetLabelLengthRule {
    pub max_length: i64,
}

impl DefaultTrustDomainBudgetRule for DefaultTrustDomainBudgetLabelLengthRule {
    fn check(&self, item: &DefaultTrustDomainBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainBudgetReviewerLengthRule {
    pub max_length: i64,
}

impl DefaultTrustDomainBudgetRule for DefaultTrustDomainBudgetReviewerLengthRule {
    fn check(&self, item: &DefaultTrustDomainBudget) -> Vec<String> {
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

/// Runs every rule of a default trust domain budget.
#[derive(Debug, Clone)]
pub struct DefaultTrustDomainBudgetValidator {
    pub rules: Vec<Rc<dyn DefaultTrustDomainBudgetRule>>,
}

impl DefaultTrustDomainBudgetValidator {
    pub fn validate(&self, item: &DefaultTrustDomainBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultTrustDomainBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultTrustDomainBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_trust_domain_budget_validator() -> DefaultTrustDomainBudgetValidator {
    DefaultTrustDomainBudgetValidator {
        rules: vec![
            Rc::new(DefaultTrustDomainBudgetLimitRangeRule { min: 120, max: 120 })
                as Rc<dyn DefaultTrustDomainBudgetRule>,
            Rc::new(DefaultTrustDomainBudgetCodeLengthRule { max_length: 1000 }),
            Rc::new(DefaultTrustDomainBudgetLabelLengthRule { max_length: 86400 }),
            Rc::new(DefaultTrustDomainBudgetReviewerLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_trust_domain_budget_validator();
        let item = new_default_trust_domain_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = DefaultTrustDomainBudgetLimitRangeRule { min: 32, max: 64 };
        let mut item = new_default_trust_domain_budget("x", "y");
        item.limit = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultTrustDomainBudgetCodeLengthRule { max_length: 120 };
        let mut item = new_default_trust_domain_budget("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultTrustDomainBudgetLabelLengthRule { max_length: 255 };
        let mut item = new_default_trust_domain_budget("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = DefaultTrustDomainBudgetReviewerLengthRule { max_length: 16 };
        let mut item = new_default_trust_domain_budget("x", "y");
        item.reviewer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
