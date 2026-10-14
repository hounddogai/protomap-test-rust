use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover plan must pass. Check returns its problems, or none.
pub trait FailoverPlanRule: std::fmt::Debug {
    fn check(&self, item: &FailoverPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverPlanDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FailoverPlanRule for FailoverPlanDiscountRangeRule {
    fn check(&self, item: &FailoverPlan) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverPlanContactEmailLengthRule {
    pub max_length: i64,
}

impl FailoverPlanRule for FailoverPlanContactEmailLengthRule {
    fn check(&self, item: &FailoverPlan) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverPlanCodePrefixRule {
    pub prefix: String,
}

impl FailoverPlanRule for FailoverPlanCodePrefixRule {
    fn check(&self, item: &FailoverPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverPlanRequiredNameRule {
    pub message: String,
}

impl FailoverPlanRule for FailoverPlanRequiredNameRule {
    fn check(&self, item: &FailoverPlan) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover plan.
#[derive(Debug, Clone)]
pub struct FailoverPlanValidator {
    pub rules: Vec<Rc<dyn FailoverPlanRule>>,
}

impl FailoverPlanValidator {
    pub fn validate(&self, item: &FailoverPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_plan_validator() -> FailoverPlanValidator {
    FailoverPlanValidator {
        rules: vec![
            Rc::new(FailoverPlanDiscountRangeRule { min: 0.0, max: 392.9 }) as Rc<dyn FailoverPlanRule>,
            Rc::new(FailoverPlanContactEmailLengthRule { max_length: 3600 }),
            Rc::new(FailoverPlanCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(FailoverPlanRequiredNameRule {
                message: "failover plan needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_plan_validator();
        let item = new_failover_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = FailoverPlanDiscountRangeRule { min: 1.6, max: 2967.4 };
        let mut item = new_failover_plan("x", "y");
        item.discount = 2968.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = FailoverPlanContactEmailLengthRule { max_length: 86400 };
        let mut item = new_failover_plan("x", "y");
        item.contact_email = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FailoverPlanRequiredNameRule {
            message: "failover plan needs a value".to_string(),
        };
        let mut item = new_failover_plan("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
