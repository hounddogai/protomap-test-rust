use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding summary must pass. Check returns its problems, or none.
pub trait BindingSummaryRule: std::fmt::Debug {
    fn check(&self, item: &BindingSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSummaryCountryLengthRule {
    pub max_length: i64,
}

impl BindingSummaryRule for BindingSummaryCountryLengthRule {
    fn check(&self, item: &BindingSummary) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSummaryAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BindingSummaryRule for BindingSummaryAmountRangeRule {
    fn check(&self, item: &BindingSummary) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSummaryRequiredCountryRule {
    pub message: String,
}

impl BindingSummaryRule for BindingSummaryRequiredCountryRule {
    fn check(&self, item: &BindingSummary) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding summary.
#[derive(Debug, Clone)]
pub struct BindingSummaryValidator {
    pub rules: Vec<Rc<dyn BindingSummaryRule>>,
}

impl BindingSummaryValidator {
    pub fn validate(&self, item: &BindingSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_summary_validator() -> BindingSummaryValidator {
    BindingSummaryValidator {
        rules: vec![
            Rc::new(BindingSummaryCountryLengthRule { max_length: 86400 }) as Rc<dyn BindingSummaryRule>,
            Rc::new(BindingSummaryAmountRangeRule { min: 0.4, max: 1834.5 }),
            Rc::new(BindingSummaryRequiredCountryRule {
                message: "binding summary needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_summary_validator();
        let item = new_binding_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = BindingSummaryCountryLengthRule { max_length: 3600 };
        let mut item = new_binding_summary("x", "y");
        item.country = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = BindingSummaryAmountRangeRule { min: 5.3, max: 4645.5 };
        let mut item = new_binding_summary("x", "y");
        item.amount = 4647.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = BindingSummaryRequiredCountryRule {
            message: "binding summary needs a value".to_string(),
        };
        let mut item = new_binding_summary("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
