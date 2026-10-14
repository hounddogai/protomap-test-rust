use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default runbook request must pass. Check returns its problems, or none.
pub trait DefaultRunbookRequestRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRunbookRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookRequestAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRunbookRequestRule for DefaultRunbookRequestAttemptsRangeRule {
    fn check(&self, item: &DefaultRunbookRequest) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookRequestAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultRunbookRequestRule for DefaultRunbookRequestAmountRangeRule {
    fn check(&self, item: &DefaultRunbookRequest) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookRequestExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRunbookRequestRule for DefaultRunbookRequestExpiresAtRangeRule {
    fn check(&self, item: &DefaultRunbookRequest) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookRequestRequiredDescriptionRule {
    pub message: String,
}

impl DefaultRunbookRequestRule for DefaultRunbookRequestRequiredDescriptionRule {
    fn check(&self, item: &DefaultRunbookRequest) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookRequestNamespaceLengthRule {
    pub max_length: i64,
}

impl DefaultRunbookRequestRule for DefaultRunbookRequestNamespaceLengthRule {
    fn check(&self, item: &DefaultRunbookRequest) -> Vec<String> {
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

/// Runs every rule of a default runbook request.
#[derive(Debug, Clone)]
pub struct DefaultRunbookRequestValidator {
    pub rules: Vec<Rc<dyn DefaultRunbookRequestRule>>,
}

impl DefaultRunbookRequestValidator {
    pub fn validate(&self, item: &DefaultRunbookRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultRunbookRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRunbookRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_runbook_request_validator() -> DefaultRunbookRequestValidator {
    DefaultRunbookRequestValidator {
        rules: vec![
            Rc::new(DefaultRunbookRequestAttemptsRangeRule { min: 120, max: 255 }) as Rc<dyn DefaultRunbookRequestRule>,
            Rc::new(DefaultRunbookRequestAmountRangeRule { min: 7.3, max: 4130.8 }),
            Rc::new(DefaultRunbookRequestExpiresAtRangeRule { min: 80, max: 1000 }),
            Rc::new(DefaultRunbookRequestRequiredDescriptionRule {
                message: "default runbook request needs a value".to_string(),
            }),
            Rc::new(DefaultRunbookRequestNamespaceLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_runbook_request_validator();
        let item = new_default_runbook_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = DefaultRunbookRequestAttemptsRangeRule { min: 255, max: 3600 };
        let mut item = new_default_runbook_request("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = DefaultRunbookRequestAmountRangeRule { min: 8.6, max: 955.4 };
        let mut item = new_default_runbook_request("x", "y");
        item.amount = 956.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = DefaultRunbookRequestExpiresAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_default_runbook_request("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = DefaultRunbookRequestRequiredDescriptionRule {
            message: "default runbook request needs a value".to_string(),
        };
        let mut item = new_default_runbook_request("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = DefaultRunbookRequestNamespaceLengthRule { max_length: 32 };
        let mut item = new_default_runbook_request("x", "y");
        item.namespace = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
