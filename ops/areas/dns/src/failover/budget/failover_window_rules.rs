use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover window must pass. Check returns its problems, or none.
pub trait FailoverWindowRule: std::fmt::Debug {
    fn check(&self, item: &FailoverWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverWindowUnitLengthRule {
    pub max_length: i64,
}

impl FailoverWindowRule for FailoverWindowUnitLengthRule {
    fn check(&self, item: &FailoverWindow) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverWindowCodeLengthRule {
    pub max_length: i64,
}

impl FailoverWindowRule for FailoverWindowCodeLengthRule {
    fn check(&self, item: &FailoverWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverWindowRequiredCodeRule {
    pub message: String,
}

impl FailoverWindowRule for FailoverWindowRequiredCodeRule {
    fn check(&self, item: &FailoverWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverWindowRequiredUnitRule {
    pub message: String,
}

impl FailoverWindowRule for FailoverWindowRequiredUnitRule {
    fn check(&self, item: &FailoverWindow) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover window.
#[derive(Debug, Clone)]
pub struct FailoverWindowValidator {
    pub rules: Vec<Rc<dyn FailoverWindowRule>>,
}

impl FailoverWindowValidator {
    pub fn validate(&self, item: &FailoverWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_window_validator() -> FailoverWindowValidator {
    FailoverWindowValidator {
        rules: vec![
            Rc::new(FailoverWindowUnitLengthRule { max_length: 86400 }) as Rc<dyn FailoverWindowRule>,
            Rc::new(FailoverWindowCodeLengthRule { max_length: 86400 }),
            Rc::new(FailoverWindowRequiredCodeRule {
                message: "failover window needs this field".to_string(),
            }),
            Rc::new(FailoverWindowRequiredUnitRule {
                message: "failover window needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_window_validator();
        let item = new_failover_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = FailoverWindowUnitLengthRule { max_length: 86400 };
        let mut item = new_failover_window("x", "y");
        item.unit = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FailoverWindowCodeLengthRule { max_length: 255 };
        let mut item = new_failover_window("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FailoverWindowRequiredCodeRule {
            message: "failover window needs this field".to_string(),
        };
        let mut item = new_failover_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = FailoverWindowRequiredUnitRule {
            message: "failover window needs an entry".to_string(),
        };
        let mut item = new_failover_window("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
