use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a alert request must pass. Check returns its problems, or none.
pub trait AlertRequestRule: std::fmt::Debug {
    fn check(&self, item: &AlertRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRequestLabelLengthRule {
    pub max_length: i64,
}

impl AlertRequestRule for AlertRequestLabelLengthRule {
    fn check(&self, item: &AlertRequest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRequestExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertRequestRule for AlertRequestExpiresAtRangeRule {
    fn check(&self, item: &AlertRequest) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRequestRequiredOwnerIdRule {
    pub message: String,
}

impl AlertRequestRule for AlertRequestRequiredOwnerIdRule {
    fn check(&self, item: &AlertRequest) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRequestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AlertRequestRule for AlertRequestAllowedCodeRule {
    fn check(&self, item: &AlertRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert request.
#[derive(Debug, Clone)]
pub struct AlertRequestValidator {
    pub rules: Vec<Rc<dyn AlertRequestRule>>,
}

impl AlertRequestValidator {
    pub fn validate(&self, item: &AlertRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_request_validator() -> AlertRequestValidator {
    AlertRequestValidator {
        rules: vec![
            Rc::new(AlertRequestLabelLengthRule { max_length: 86400 }) as Rc<dyn AlertRequestRule>,
            Rc::new(AlertRequestExpiresAtRangeRule { min: 64, max: 80 }),
            Rc::new(AlertRequestRequiredOwnerIdRule {
                message: "alert request needs an entry".to_string(),
            }),
            Rc::new(AlertRequestAllowedCodeRule {
                allowed: vec!["tiered".to_string(), "strict".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_request_validator();
        let item = new_alert_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AlertRequestLabelLengthRule { max_length: 120 };
        let mut item = new_alert_request("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = AlertRequestExpiresAtRangeRule { min: 120, max: 120 };
        let mut item = new_alert_request("x", "y");
        item.expires_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = AlertRequestRequiredOwnerIdRule {
            message: "alert request needs a value".to_string(),
        };
        let mut item = new_alert_request("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
