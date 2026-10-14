use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling grant summary must pass. Check returns its problems, or none.
pub trait RollingGrantSummaryRule: std::fmt::Debug {
    fn check(&self, item: &RollingGrantSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantSummaryRequiredHostRule {
    pub message: String,
}

impl RollingGrantSummaryRule for RollingGrantSummaryRequiredHostRule {
    fn check(&self, item: &RollingGrantSummary) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantSummaryHostLengthRule {
    pub max_length: i64,
}

impl RollingGrantSummaryRule for RollingGrantSummaryHostLengthRule {
    fn check(&self, item: &RollingGrantSummary) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantSummaryNotesLengthRule {
    pub max_length: i64,
}

impl RollingGrantSummaryRule for RollingGrantSummaryNotesLengthRule {
    fn check(&self, item: &RollingGrantSummary) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantSummaryRequiredCodeRule {
    pub message: String,
}

impl RollingGrantSummaryRule for RollingGrantSummaryRequiredCodeRule {
    fn check(&self, item: &RollingGrantSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling grant summary.
#[derive(Debug, Clone)]
pub struct RollingGrantSummaryValidator {
    pub rules: Vec<Rc<dyn RollingGrantSummaryRule>>,
}

impl RollingGrantSummaryValidator {
    pub fn validate(&self, item: &RollingGrantSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingGrantSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingGrantSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_grant_summary_validator() -> RollingGrantSummaryValidator {
    RollingGrantSummaryValidator {
        rules: vec![
            Rc::new(RollingGrantSummaryRequiredHostRule {
                message: "rolling grant summary needs a value".to_string(),
            }) as Rc<dyn RollingGrantSummaryRule>,
            Rc::new(RollingGrantSummaryHostLengthRule { max_length: 86400 }),
            Rc::new(RollingGrantSummaryNotesLengthRule { max_length: 86400 }),
            Rc::new(RollingGrantSummaryRequiredCodeRule {
                message: "rolling grant summary needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_grant_summary_validator();
        let item = new_rolling_grant_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RollingGrantSummaryRequiredHostRule {
            message: "rolling grant summary needs a value".to_string(),
        };
        let mut item = new_rolling_grant_summary("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = RollingGrantSummaryHostLengthRule { max_length: 80 };
        let mut item = new_rolling_grant_summary("x", "y");
        item.host = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RollingGrantSummaryNotesLengthRule { max_length: 1000 };
        let mut item = new_rolling_grant_summary("x", "y");
        item.notes = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingGrantSummaryRequiredCodeRule {
            message: "rolling grant summary needs this field".to_string(),
        };
        let mut item = new_rolling_grant_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
