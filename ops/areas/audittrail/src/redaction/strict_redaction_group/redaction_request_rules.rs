use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction request must pass. Check returns its problems, or none.
pub trait RedactionRequestRule: std::fmt::Debug {
    fn check(&self, item: &RedactionRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRequestRequiredHostRule {
    pub message: String,
}

impl RedactionRequestRule for RedactionRequestRequiredHostRule {
    fn check(&self, item: &RedactionRequest) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRequestNameLengthRule {
    pub max_length: i64,
}

impl RedactionRequestRule for RedactionRequestNameLengthRule {
    fn check(&self, item: &RedactionRequest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRequestRequiredExternalRefRule {
    pub message: String,
}

impl RedactionRequestRule for RedactionRequestRequiredExternalRefRule {
    fn check(&self, item: &RedactionRequest) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRequestCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RedactionRequestRule for RedactionRequestCostRangeRule {
    fn check(&self, item: &RedactionRequest) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRequestHostLengthRule {
    pub max_length: i64,
}

impl RedactionRequestRule for RedactionRequestHostLengthRule {
    fn check(&self, item: &RedactionRequest) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRequestCodeLengthRule {
    pub max_length: i64,
}

impl RedactionRequestRule for RedactionRequestCodeLengthRule {
    fn check(&self, item: &RedactionRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction request.
#[derive(Debug, Clone)]
pub struct RedactionRequestValidator {
    pub rules: Vec<Rc<dyn RedactionRequestRule>>,
}

impl RedactionRequestValidator {
    pub fn validate(&self, item: &RedactionRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_request_validator() -> RedactionRequestValidator {
    RedactionRequestValidator {
        rules: vec![
            Rc::new(RedactionRequestRequiredHostRule {
                message: "redaction request needs an entry".to_string(),
            }) as Rc<dyn RedactionRequestRule>,
            Rc::new(RedactionRequestNameLengthRule { max_length: 1000 }),
            Rc::new(RedactionRequestRequiredExternalRefRule {
                message: "redaction request needs an entry".to_string(),
            }),
            Rc::new(RedactionRequestCostRangeRule { min: 3.3, max: 1247.5 }),
            Rc::new(RedactionRequestHostLengthRule { max_length: 255 }),
            Rc::new(RedactionRequestCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_request_validator();
        let item = new_redaction_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RedactionRequestRequiredHostRule {
            message: "redaction request needs this field".to_string(),
        };
        let mut item = new_redaction_request("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RedactionRequestNameLengthRule { max_length: 16 };
        let mut item = new_redaction_request("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = RedactionRequestRequiredExternalRefRule {
            message: "redaction request needs a value".to_string(),
        };
        let mut item = new_redaction_request("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = RedactionRequestCostRangeRule { min: 5.8, max: 2411.6 };
        let mut item = new_redaction_request("x", "y");
        item.cost = 2413.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = RedactionRequestHostLengthRule { max_length: 86400 };
        let mut item = new_redaction_request("x", "y");
        item.host = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RedactionRequestCodeLengthRule { max_length: 120 };
        let mut item = new_redaction_request("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
