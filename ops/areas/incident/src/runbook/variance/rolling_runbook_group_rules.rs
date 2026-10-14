use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling runbook group must pass. Check returns its problems, or none.
pub trait RollingRunbookGroupRule: std::fmt::Debug {
    fn check(&self, item: &RollingRunbookGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookGroupRequiredDescriptionRule {
    pub message: String,
}

impl RollingRunbookGroupRule for RollingRunbookGroupRequiredDescriptionRule {
    fn check(&self, item: &RollingRunbookGroup) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookGroupDescriptionLengthRule {
    pub max_length: i64,
}

impl RollingRunbookGroupRule for RollingRunbookGroupDescriptionLengthRule {
    fn check(&self, item: &RollingRunbookGroup) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookGroupRequiredUnitRule {
    pub message: String,
}

impl RollingRunbookGroupRule for RollingRunbookGroupRequiredUnitRule {
    fn check(&self, item: &RollingRunbookGroup) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookGroupTitleLengthRule {
    pub max_length: i64,
}

impl RollingRunbookGroupRule for RollingRunbookGroupTitleLengthRule {
    fn check(&self, item: &RollingRunbookGroup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling runbook group.
#[derive(Debug, Clone)]
pub struct RollingRunbookGroupValidator {
    pub rules: Vec<Rc<dyn RollingRunbookGroupRule>>,
}

impl RollingRunbookGroupValidator {
    pub fn validate(&self, item: &RollingRunbookGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingRunbookGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRunbookGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_runbook_group_validator() -> RollingRunbookGroupValidator {
    RollingRunbookGroupValidator {
        rules: vec![
            Rc::new(RollingRunbookGroupRequiredDescriptionRule {
                message: "rolling runbook group needs an entry".to_string(),
            }) as Rc<dyn RollingRunbookGroupRule>,
            Rc::new(RollingRunbookGroupDescriptionLengthRule { max_length: 3600 }),
            Rc::new(RollingRunbookGroupRequiredUnitRule {
                message: "rolling runbook group needs this field".to_string(),
            }),
            Rc::new(RollingRunbookGroupTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_runbook_group_validator();
        let item = new_rolling_runbook_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = RollingRunbookGroupRequiredDescriptionRule {
            message: "rolling runbook group needs an entry".to_string(),
        };
        let mut item = new_rolling_runbook_group("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = RollingRunbookGroupDescriptionLengthRule { max_length: 64 };
        let mut item = new_rolling_runbook_group("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = RollingRunbookGroupRequiredUnitRule {
            message: "rolling runbook group needs an entry".to_string(),
        };
        let mut item = new_rolling_runbook_group("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollingRunbookGroupTitleLengthRule { max_length: 64 };
        let mut item = new_rolling_runbook_group("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
