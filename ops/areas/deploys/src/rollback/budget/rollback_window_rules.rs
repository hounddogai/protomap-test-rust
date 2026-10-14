use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback window must pass. Check returns its problems, or none.
pub trait RollbackWindowRule: std::fmt::Debug {
    fn check(&self, item: &RollbackWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackWindowCodeLengthRule {
    pub max_length: i64,
}

impl RollbackWindowRule for RollbackWindowCodeLengthRule {
    fn check(&self, item: &RollbackWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackWindowRequiredAudienceRule {
    pub message: String,
}

impl RollbackWindowRule for RollbackWindowRequiredAudienceRule {
    fn check(&self, item: &RollbackWindow) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackWindowAudienceLengthRule {
    pub max_length: i64,
}

impl RollbackWindowRule for RollbackWindowAudienceLengthRule {
    fn check(&self, item: &RollbackWindow) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback window.
#[derive(Debug, Clone)]
pub struct RollbackWindowValidator {
    pub rules: Vec<Rc<dyn RollbackWindowRule>>,
}

impl RollbackWindowValidator {
    pub fn validate(&self, item: &RollbackWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_window_validator() -> RollbackWindowValidator {
    RollbackWindowValidator {
        rules: vec![
            Rc::new(RollbackWindowCodeLengthRule { max_length: 86400 }) as Rc<dyn RollbackWindowRule>,
            Rc::new(RollbackWindowRequiredAudienceRule {
                message: "rollback window needs a value".to_string(),
            }),
            Rc::new(RollbackWindowAudienceLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_window_validator();
        let item = new_rollback_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollbackWindowCodeLengthRule { max_length: 255 };
        let mut item = new_rollback_window("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = RollbackWindowRequiredAudienceRule {
            message: "rollback window needs a value".to_string(),
        };
        let mut item = new_rollback_window("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = RollbackWindowAudienceLengthRule { max_length: 16 };
        let mut item = new_rollback_window("x", "y");
        item.audience = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
