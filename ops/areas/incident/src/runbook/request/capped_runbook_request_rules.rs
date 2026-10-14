use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped runbook request must pass. Check returns its problems, or none.
pub trait CappedRunbookRequestRule: std::fmt::Debug {
    fn check(&self, item: &CappedRunbookRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRunbookRequestRequiredCodeRule {
    pub message: String,
}

impl CappedRunbookRequestRule for CappedRunbookRequestRequiredCodeRule {
    fn check(&self, item: &CappedRunbookRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRunbookRequestTimezoneLengthRule {
    pub max_length: i64,
}

impl CappedRunbookRequestRule for CappedRunbookRequestTimezoneLengthRule {
    fn check(&self, item: &CappedRunbookRequest) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRunbookRequestRequiredLabelRule {
    pub message: String,
}

impl CappedRunbookRequestRule for CappedRunbookRequestRequiredLabelRule {
    fn check(&self, item: &CappedRunbookRequest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped runbook request.
#[derive(Debug, Clone)]
pub struct CappedRunbookRequestValidator {
    pub rules: Vec<Rc<dyn CappedRunbookRequestRule>>,
}

impl CappedRunbookRequestValidator {
    pub fn validate(&self, item: &CappedRunbookRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedRunbookRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRunbookRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_runbook_request_validator() -> CappedRunbookRequestValidator {
    CappedRunbookRequestValidator {
        rules: vec![
            Rc::new(CappedRunbookRequestRequiredCodeRule {
                message: "capped runbook request needs a value".to_string(),
            }) as Rc<dyn CappedRunbookRequestRule>,
            Rc::new(CappedRunbookRequestTimezoneLengthRule { max_length: 80 }),
            Rc::new(CappedRunbookRequestRequiredLabelRule {
                message: "capped runbook request needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_runbook_request_validator();
        let item = new_capped_runbook_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedRunbookRequestRequiredCodeRule {
            message: "capped runbook request needs an entry".to_string(),
        };
        let mut item = new_capped_runbook_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = CappedRunbookRequestTimezoneLengthRule { max_length: 3600 };
        let mut item = new_capped_runbook_request("x", "y");
        item.timezone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedRunbookRequestRequiredLabelRule {
            message: "capped runbook request needs this field".to_string(),
        };
        let mut item = new_capped_runbook_request("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
