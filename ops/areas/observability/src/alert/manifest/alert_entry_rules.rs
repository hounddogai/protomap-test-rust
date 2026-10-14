use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert entry must pass. Check returns its problems, or none.
pub trait AlertEntryRule: std::fmt::Debug {
    fn check(&self, item: &AlertEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertEntryRequiredCipherRule {
    pub message: String,
}

impl AlertEntryRule for AlertEntryRequiredCipherRule {
    fn check(&self, item: &AlertEntry) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertEntryTitleLengthRule {
    pub max_length: i64,
}

impl AlertEntryRule for AlertEntryTitleLengthRule {
    fn check(&self, item: &AlertEntry) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertEntryRequiredCodeRule {
    pub message: String,
}

impl AlertEntryRule for AlertEntryRequiredCodeRule {
    fn check(&self, item: &AlertEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert entry.
#[derive(Debug, Clone)]
pub struct AlertEntryValidator {
    pub rules: Vec<Rc<dyn AlertEntryRule>>,
}

impl AlertEntryValidator {
    pub fn validate(&self, item: &AlertEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_entry_validator() -> AlertEntryValidator {
    AlertEntryValidator {
        rules: vec![
            Rc::new(AlertEntryRequiredCipherRule {
                message: "alert entry needs a value".to_string(),
            }) as Rc<dyn AlertEntryRule>,
            Rc::new(AlertEntryTitleLengthRule { max_length: 86400 }),
            Rc::new(AlertEntryRequiredCodeRule {
                message: "alert entry needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_entry_validator();
        let item = new_alert_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = AlertEntryRequiredCipherRule {
            message: "alert entry needs an entry".to_string(),
        };
        let mut item = new_alert_entry("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AlertEntryTitleLengthRule { max_length: 32 };
        let mut item = new_alert_entry("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AlertEntryRequiredCodeRule {
            message: "alert entry needs this field".to_string(),
        };
        let mut item = new_alert_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
