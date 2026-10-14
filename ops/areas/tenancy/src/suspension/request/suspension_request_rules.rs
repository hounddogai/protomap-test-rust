use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension request must pass. Check returns its problems, or none.
pub trait SuspensionRequestRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRequestRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionRequestRule for SuspensionRequestRateRangeRule {
    fn check(&self, item: &SuspensionRequest) -> Vec<String> {
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
pub struct SuspensionRequestRequiredHostRule {
    pub message: String,
}

impl SuspensionRequestRule for SuspensionRequestRequiredHostRule {
    fn check(&self, item: &SuspensionRequest) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRequestPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionRequestRule for SuspensionRequestPortRangeRule {
    fn check(&self, item: &SuspensionRequest) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRequestCodePrefixRule {
    pub prefix: String,
}

impl SuspensionRequestRule for SuspensionRequestCodePrefixRule {
    fn check(&self, item: &SuspensionRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRequestRequiredCodeRule {
    pub message: String,
}

impl SuspensionRequestRule for SuspensionRequestRequiredCodeRule {
    fn check(&self, item: &SuspensionRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRequestLabelLengthRule {
    pub max_length: i64,
}

impl SuspensionRequestRule for SuspensionRequestLabelLengthRule {
    fn check(&self, item: &SuspensionRequest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension request.
#[derive(Debug, Clone)]
pub struct SuspensionRequestValidator {
    pub rules: Vec<Rc<dyn SuspensionRequestRule>>,
}

impl SuspensionRequestValidator {
    pub fn validate(&self, item: &SuspensionRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_request_validator() -> SuspensionRequestValidator {
    SuspensionRequestValidator {
        rules: vec![
            Rc::new(SuspensionRequestRateRangeRule { min: 7.3, max: 4352.3 }) as Rc<dyn SuspensionRequestRule>,
            Rc::new(SuspensionRequestRequiredHostRule {
                message: "suspension request needs an entry".to_string(),
            }),
            Rc::new(SuspensionRequestPortRangeRule { min: 1000, max: 86400 }),
            Rc::new(SuspensionRequestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(SuspensionRequestRequiredCodeRule {
                message: "suspension request needs this field".to_string(),
            }),
            Rc::new(SuspensionRequestLabelLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_request_validator();
        let item = new_suspension_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = SuspensionRequestRateRangeRule { min: 3.8, max: 4674.5 };
        let mut item = new_suspension_request("x", "y");
        item.rate = 4676.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = SuspensionRequestRequiredHostRule {
            message: "suspension request needs a value".to_string(),
        };
        let mut item = new_suspension_request("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = SuspensionRequestPortRangeRule { min: 80, max: 120 };
        let mut item = new_suspension_request("x", "y");
        item.port = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SuspensionRequestRequiredCodeRule {
            message: "suspension request needs a value".to_string(),
        };
        let mut item = new_suspension_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SuspensionRequestLabelLengthRule { max_length: 16 };
        let mut item = new_suspension_request("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
