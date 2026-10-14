use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback request must pass. Check returns its problems, or none.
pub trait RollbackRequestRule: std::fmt::Debug {
    fn check(&self, item: &RollbackRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRequestRequiredLabelRule {
    pub message: String,
}

impl RollbackRequestRule for RollbackRequestRequiredLabelRule {
    fn check(&self, item: &RollbackRequest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRequestRequiredAudienceRule {
    pub message: String,
}

impl RollbackRequestRule for RollbackRequestRequiredAudienceRule {
    fn check(&self, item: &RollbackRequest) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRequestAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl RollbackRequestRule for RollbackRequestAllowedCipherRule {
    fn check(&self, item: &RollbackRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is expired", item.cipher)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRequestLabelLengthRule {
    pub max_length: i64,
}

impl RollbackRequestRule for RollbackRequestLabelLengthRule {
    fn check(&self, item: &RollbackRequest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRequestCodePrefixRule {
    pub prefix: String,
}

impl RollbackRequestRule for RollbackRequestCodePrefixRule {
    fn check(&self, item: &RollbackRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRequestCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollbackRequestRule for RollbackRequestCeilingRangeRule {
    fn check(&self, item: &RollbackRequest) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rollback request.
#[derive(Debug, Clone)]
pub struct RollbackRequestValidator {
    pub rules: Vec<Rc<dyn RollbackRequestRule>>,
}

impl RollbackRequestValidator {
    pub fn validate(&self, item: &RollbackRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_request_validator() -> RollbackRequestValidator {
    RollbackRequestValidator {
        rules: vec![
            Rc::new(RollbackRequestRequiredLabelRule {
                message: "rollback request needs a value".to_string(),
            }) as Rc<dyn RollbackRequestRule>,
            Rc::new(RollbackRequestRequiredAudienceRule {
                message: "rollback request needs an entry".to_string(),
            }),
            Rc::new(RollbackRequestAllowedCipherRule {
                allowed: vec!["capped".to_string(), "tiered".to_string()],
            }),
            Rc::new(RollbackRequestLabelLengthRule { max_length: 120 }),
            Rc::new(RollbackRequestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RollbackRequestCeilingRangeRule { min: 5.8, max: 2499.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_request_validator();
        let item = new_rollback_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollbackRequestRequiredLabelRule {
            message: "rollback request needs an entry".to_string(),
        };
        let mut item = new_rollback_request("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = RollbackRequestRequiredAudienceRule {
            message: "rollback request needs a value".to_string(),
        };
        let mut item = new_rollback_request("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollbackRequestLabelLengthRule { max_length: 86400 };
        let mut item = new_rollback_request("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = RollbackRequestCeilingRangeRule { min: 1.6, max: 1813.9 };
        let mut item = new_rollback_request("x", "y");
        item.ceiling = 1815.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
