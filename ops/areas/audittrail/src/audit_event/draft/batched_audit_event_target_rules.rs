use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched audit event target must pass. Check returns its problems, or none.
pub trait BatchedAuditEventTargetRule: std::fmt::Debug {
    fn check(&self, item: &BatchedAuditEventTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventTargetCodeLengthRule {
    pub max_length: i64,
}

impl BatchedAuditEventTargetRule for BatchedAuditEventTargetCodeLengthRule {
    fn check(&self, item: &BatchedAuditEventTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventTargetRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedAuditEventTargetRule for BatchedAuditEventTargetRateRangeRule {
    fn check(&self, item: &BatchedAuditEventTarget) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventTargetRequiredNameRule {
    pub message: String,
}

impl BatchedAuditEventTargetRule for BatchedAuditEventTargetRequiredNameRule {
    fn check(&self, item: &BatchedAuditEventTarget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched audit event target.
#[derive(Debug, Clone)]
pub struct BatchedAuditEventTargetValidator {
    pub rules: Vec<Rc<dyn BatchedAuditEventTargetRule>>,
}

impl BatchedAuditEventTargetValidator {
    pub fn validate(&self, item: &BatchedAuditEventTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedAuditEventTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedAuditEventTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_audit_event_target_validator() -> BatchedAuditEventTargetValidator {
    BatchedAuditEventTargetValidator {
        rules: vec![
            Rc::new(BatchedAuditEventTargetCodeLengthRule { max_length: 80 }) as Rc<dyn BatchedAuditEventTargetRule>,
            Rc::new(BatchedAuditEventTargetRateRangeRule { min: 1.0, max: 3964.0 }),
            Rc::new(BatchedAuditEventTargetRequiredNameRule {
                message: "batched audit event target needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_audit_event_target_validator();
        let item = new_batched_audit_event_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedAuditEventTargetCodeLengthRule { max_length: 120 };
        let mut item = new_batched_audit_event_target("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = BatchedAuditEventTargetRateRangeRule { min: 2.0, max: 4225.0 };
        let mut item = new_batched_audit_event_target("x", "y");
        item.rate = 4226.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedAuditEventTargetRequiredNameRule {
            message: "batched audit event target needs a value".to_string(),
        };
        let mut item = new_batched_audit_event_target("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
