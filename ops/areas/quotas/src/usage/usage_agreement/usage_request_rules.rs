use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage request must pass. Check returns its problems, or none.
pub trait UsageRequestRule: std::fmt::Debug {
    fn check(&self, item: &UsageRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRequestLabelLengthRule {
    pub max_length: i64,
}

impl UsageRequestRule for UsageRequestLabelLengthRule {
    fn check(&self, item: &UsageRequest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRequestNameLengthRule {
    pub max_length: i64,
}

impl UsageRequestRule for UsageRequestNameLengthRule {
    fn check(&self, item: &UsageRequest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRequestCodeLengthRule {
    pub max_length: i64,
}

impl UsageRequestRule for UsageRequestCodeLengthRule {
    fn check(&self, item: &UsageRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRequestRequiredIssuerRule {
    pub message: String,
}

impl UsageRequestRule for UsageRequestRequiredIssuerRule {
    fn check(&self, item: &UsageRequest) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage request.
#[derive(Debug, Clone)]
pub struct UsageRequestValidator {
    pub rules: Vec<Rc<dyn UsageRequestRule>>,
}

impl UsageRequestValidator {
    pub fn validate(&self, item: &UsageRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_request_validator() -> UsageRequestValidator {
    UsageRequestValidator {
        rules: vec![
            Rc::new(UsageRequestLabelLengthRule { max_length: 32 }) as Rc<dyn UsageRequestRule>,
            Rc::new(UsageRequestNameLengthRule { max_length: 80 }),
            Rc::new(UsageRequestCodeLengthRule { max_length: 120 }),
            Rc::new(UsageRequestRequiredIssuerRule {
                message: "usage request needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_request_validator();
        let item = new_usage_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = UsageRequestLabelLengthRule { max_length: 120 };
        let mut item = new_usage_request("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = UsageRequestNameLengthRule { max_length: 3600 };
        let mut item = new_usage_request("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = UsageRequestCodeLengthRule { max_length: 32 };
        let mut item = new_usage_request("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = UsageRequestRequiredIssuerRule {
            message: "usage request needs an entry".to_string(),
        };
        let mut item = new_usage_request("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
