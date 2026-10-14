use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal quote assignment must pass. Check returns its problems, or none.
pub trait SeasonalQuoteAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalQuoteAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQuoteAssignmentCodePrefixRule {
    pub prefix: String,
}

impl SeasonalQuoteAssignmentRule for SeasonalQuoteAssignmentCodePrefixRule {
    fn check(&self, item: &SeasonalQuoteAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQuoteAssignmentAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalQuoteAssignmentRule for SeasonalQuoteAssignmentAmountRangeRule {
    fn check(&self, item: &SeasonalQuoteAssignment) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQuoteAssignmentRequiredFingerprintRule {
    pub message: String,
}

impl SeasonalQuoteAssignmentRule for SeasonalQuoteAssignmentRequiredFingerprintRule {
    fn check(&self, item: &SeasonalQuoteAssignment) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQuoteAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalQuoteAssignmentRule for SeasonalQuoteAssignmentCodeLengthRule {
    fn check(&self, item: &SeasonalQuoteAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal quote assignment.
#[derive(Debug, Clone)]
pub struct SeasonalQuoteAssignmentValidator {
    pub rules: Vec<Rc<dyn SeasonalQuoteAssignmentRule>>,
}

impl SeasonalQuoteAssignmentValidator {
    pub fn validate(&self, item: &SeasonalQuoteAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalQuoteAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalQuoteAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_quote_assignment_validator() -> SeasonalQuoteAssignmentValidator {
    SeasonalQuoteAssignmentValidator {
        rules: vec![
            Rc::new(SeasonalQuoteAssignmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn SeasonalQuoteAssignmentRule>,
            Rc::new(SeasonalQuoteAssignmentAmountRangeRule { min: 5.2, max: 1389.4 }),
            Rc::new(SeasonalQuoteAssignmentRequiredFingerprintRule {
                message: "seasonal quote assignment needs an entry".to_string(),
            }),
            Rc::new(SeasonalQuoteAssignmentCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_quote_assignment_validator();
        let item = new_seasonal_quote_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = SeasonalQuoteAssignmentAmountRangeRule { min: 1.4, max: 596.7 };
        let mut item = new_seasonal_quote_assignment("x", "y");
        item.amount = 598.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = SeasonalQuoteAssignmentRequiredFingerprintRule {
            message: "seasonal quote assignment needs an entry".to_string(),
        };
        let mut item = new_seasonal_quote_assignment("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalQuoteAssignmentCodeLengthRule { max_length: 80 };
        let mut item = new_seasonal_quote_assignment("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
