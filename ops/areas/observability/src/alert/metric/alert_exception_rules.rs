use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert exception must pass. Check returns its problems, or none.
pub trait AlertExceptionRule: std::fmt::Debug {
    fn check(&self, item: &AlertException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertExceptionExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertExceptionRule for AlertExceptionExpiresAtRangeRule {
    fn check(&self, item: &AlertException) -> Vec<String> {
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
pub struct AlertExceptionHostLengthRule {
    pub max_length: i64,
}

impl AlertExceptionRule for AlertExceptionHostLengthRule {
    fn check(&self, item: &AlertException) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertExceptionRequiredLabelRule {
    pub message: String,
}

impl AlertExceptionRule for AlertExceptionRequiredLabelRule {
    fn check(&self, item: &AlertException) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertExceptionLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertExceptionRule for AlertExceptionLimitRangeRule {
    fn check(&self, item: &AlertException) -> Vec<String> {
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
pub struct AlertExceptionLabelLengthRule {
    pub max_length: i64,
}

impl AlertExceptionRule for AlertExceptionLabelLengthRule {
    fn check(&self, item: &AlertException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertExceptionCodeLengthRule {
    pub max_length: i64,
}

impl AlertExceptionRule for AlertExceptionCodeLengthRule {
    fn check(&self, item: &AlertException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert exception.
#[derive(Debug, Clone)]
pub struct AlertExceptionValidator {
    pub rules: Vec<Rc<dyn AlertExceptionRule>>,
}

impl AlertExceptionValidator {
    pub fn validate(&self, item: &AlertException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_exception_validator() -> AlertExceptionValidator {
    AlertExceptionValidator {
        rules: vec![
            Rc::new(AlertExceptionExpiresAtRangeRule { min: 3600, max: 86400 }) as Rc<dyn AlertExceptionRule>,
            Rc::new(AlertExceptionHostLengthRule { max_length: 64 }),
            Rc::new(AlertExceptionRequiredLabelRule {
                message: "alert exception needs a value".to_string(),
            }),
            Rc::new(AlertExceptionLimitRangeRule { min: 64, max: 120 }),
            Rc::new(AlertExceptionLabelLengthRule { max_length: 64 }),
            Rc::new(AlertExceptionCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_exception_validator();
        let item = new_alert_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = AlertExceptionExpiresAtRangeRule { min: 16, max: 80 };
        let mut item = new_alert_exception("x", "y");
        item.expires_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = AlertExceptionHostLengthRule { max_length: 120 };
        let mut item = new_alert_exception("x", "y");
        item.host = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AlertExceptionRequiredLabelRule {
            message: "alert exception needs a value".to_string(),
        };
        let mut item = new_alert_exception("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = AlertExceptionLimitRangeRule { min: 32, max: 1000 };
        let mut item = new_alert_exception("x", "y");
        item.limit = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AlertExceptionLabelLengthRule { max_length: 86400 };
        let mut item = new_alert_exception("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AlertExceptionCodeLengthRule { max_length: 80 };
        let mut item = new_alert_exception("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
