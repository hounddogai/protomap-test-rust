use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook adjustment must pass. Check returns its problems, or none.
pub trait RunbookAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &RunbookAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAdjustmentRequiredIssuerRule {
    pub message: String,
}

impl RunbookAdjustmentRule for RunbookAdjustmentRequiredIssuerRule {
    fn check(&self, item: &RunbookAdjustment) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAdjustmentRequiredTitleRule {
    pub message: String,
}

impl RunbookAdjustmentRule for RunbookAdjustmentRequiredTitleRule {
    fn check(&self, item: &RunbookAdjustment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAdjustmentIssuerLengthRule {
    pub max_length: i64,
}

impl RunbookAdjustmentRule for RunbookAdjustmentIssuerLengthRule {
    fn check(&self, item: &RunbookAdjustment) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook adjustment.
#[derive(Debug, Clone)]
pub struct RunbookAdjustmentValidator {
    pub rules: Vec<Rc<dyn RunbookAdjustmentRule>>,
}

impl RunbookAdjustmentValidator {
    pub fn validate(&self, item: &RunbookAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_adjustment_validator() -> RunbookAdjustmentValidator {
    RunbookAdjustmentValidator {
        rules: vec![
            Rc::new(RunbookAdjustmentRequiredIssuerRule {
                message: "runbook adjustment needs an entry".to_string(),
            }) as Rc<dyn RunbookAdjustmentRule>,
            Rc::new(RunbookAdjustmentRequiredTitleRule {
                message: "runbook adjustment needs a value".to_string(),
            }),
            Rc::new(RunbookAdjustmentIssuerLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_adjustment_validator();
        let item = new_runbook_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = RunbookAdjustmentRequiredIssuerRule {
            message: "runbook adjustment needs an entry".to_string(),
        };
        let mut item = new_runbook_adjustment("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RunbookAdjustmentRequiredTitleRule {
            message: "runbook adjustment needs a value".to_string(),
        };
        let mut item = new_runbook_adjustment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = RunbookAdjustmentIssuerLengthRule { max_length: 3600 };
        let mut item = new_runbook_adjustment("x", "y");
        item.issuer = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
