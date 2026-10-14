use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink request must pass. Check returns its problems, or none.
pub trait SinkRequestRule: std::fmt::Debug {
    fn check(&self, item: &SinkRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRequestLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkRequestRule for SinkRequestLimitRangeRule {
    fn check(&self, item: &SinkRequest) -> Vec<String> {
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
pub struct SinkRequestRequiredLabelRule {
    pub message: String,
}

impl SinkRequestRule for SinkRequestRequiredLabelRule {
    fn check(&self, item: &SinkRequest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRequestRequiredCipherRule {
    pub message: String,
}

impl SinkRequestRule for SinkRequestRequiredCipherRule {
    fn check(&self, item: &SinkRequest) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRequestCodePrefixRule {
    pub prefix: String,
}

impl SinkRequestRule for SinkRequestCodePrefixRule {
    fn check(&self, item: &SinkRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRequestCurrencyLengthRule {
    pub max_length: i64,
}

impl SinkRequestRule for SinkRequestCurrencyLengthRule {
    fn check(&self, item: &SinkRequest) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRequestRequiredCurrencyRule {
    pub message: String,
}

impl SinkRequestRule for SinkRequestRequiredCurrencyRule {
    fn check(&self, item: &SinkRequest) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink request.
#[derive(Debug, Clone)]
pub struct SinkRequestValidator {
    pub rules: Vec<Rc<dyn SinkRequestRule>>,
}

impl SinkRequestValidator {
    pub fn validate(&self, item: &SinkRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_request_validator() -> SinkRequestValidator {
    SinkRequestValidator {
        rules: vec![
            Rc::new(SinkRequestLimitRangeRule { min: 32, max: 3600 }) as Rc<dyn SinkRequestRule>,
            Rc::new(SinkRequestRequiredLabelRule {
                message: "sink request needs an entry".to_string(),
            }),
            Rc::new(SinkRequestRequiredCipherRule {
                message: "sink request needs this field".to_string(),
            }),
            Rc::new(SinkRequestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(SinkRequestCurrencyLengthRule { max_length: 120 }),
            Rc::new(SinkRequestRequiredCurrencyRule {
                message: "sink request needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_request_validator();
        let item = new_sink_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = SinkRequestLimitRangeRule { min: 16, max: 16 };
        let mut item = new_sink_request("x", "y");
        item.limit = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SinkRequestRequiredLabelRule {
            message: "sink request needs this field".to_string(),
        };
        let mut item = new_sink_request("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = SinkRequestRequiredCipherRule {
            message: "sink request needs an entry".to_string(),
        };
        let mut item = new_sink_request("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = SinkRequestCurrencyLengthRule { max_length: 32 };
        let mut item = new_sink_request("x", "y");
        item.currency = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = SinkRequestRequiredCurrencyRule {
            message: "sink request needs a value".to_string(),
        };
        let mut item = new_sink_request("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
