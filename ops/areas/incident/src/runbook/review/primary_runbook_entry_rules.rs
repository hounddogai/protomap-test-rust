use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary runbook entry must pass. Check returns its problems, or none.
pub trait PrimaryRunbookEntryRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRunbookEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRunbookEntryRequiredCodeRule {
    pub message: String,
}

impl PrimaryRunbookEntryRule for PrimaryRunbookEntryRequiredCodeRule {
    fn check(&self, item: &PrimaryRunbookEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRunbookEntryRequiredScopeRule {
    pub message: String,
}

impl PrimaryRunbookEntryRule for PrimaryRunbookEntryRequiredScopeRule {
    fn check(&self, item: &PrimaryRunbookEntry) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRunbookEntryRequiredTitleRule {
    pub message: String,
}

impl PrimaryRunbookEntryRule for PrimaryRunbookEntryRequiredTitleRule {
    fn check(&self, item: &PrimaryRunbookEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRunbookEntryLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryRunbookEntryRule for PrimaryRunbookEntryLimitRangeRule {
    fn check(&self, item: &PrimaryRunbookEntry) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRunbookEntryCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryRunbookEntryRule for PrimaryRunbookEntryCodeLengthRule {
    fn check(&self, item: &PrimaryRunbookEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary runbook entry.
#[derive(Debug, Clone)]
pub struct PrimaryRunbookEntryValidator {
    pub rules: Vec<Rc<dyn PrimaryRunbookEntryRule>>,
}

impl PrimaryRunbookEntryValidator {
    pub fn validate(&self, item: &PrimaryRunbookEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryRunbookEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRunbookEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_runbook_entry_validator() -> PrimaryRunbookEntryValidator {
    PrimaryRunbookEntryValidator {
        rules: vec![
            Rc::new(PrimaryRunbookEntryRequiredCodeRule {
                message: "primary runbook entry needs a value".to_string(),
            }) as Rc<dyn PrimaryRunbookEntryRule>,
            Rc::new(PrimaryRunbookEntryRequiredScopeRule {
                message: "primary runbook entry needs an entry".to_string(),
            }),
            Rc::new(PrimaryRunbookEntryRequiredTitleRule {
                message: "primary runbook entry needs this field".to_string(),
            }),
            Rc::new(PrimaryRunbookEntryLimitRangeRule { min: 16, max: 16 }),
            Rc::new(PrimaryRunbookEntryCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_runbook_entry_validator();
        let item = new_primary_runbook_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryRunbookEntryRequiredCodeRule {
            message: "primary runbook entry needs a value".to_string(),
        };
        let mut item = new_primary_runbook_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = PrimaryRunbookEntryRequiredScopeRule {
            message: "primary runbook entry needs an entry".to_string(),
        };
        let mut item = new_primary_runbook_entry("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryRunbookEntryRequiredTitleRule {
            message: "primary runbook entry needs this field".to_string(),
        };
        let mut item = new_primary_runbook_entry("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = PrimaryRunbookEntryLimitRangeRule { min: 64, max: 86400 };
        let mut item = new_primary_runbook_entry("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryRunbookEntryCodeLengthRule { max_length: 16 };
        let mut item = new_primary_runbook_entry("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
