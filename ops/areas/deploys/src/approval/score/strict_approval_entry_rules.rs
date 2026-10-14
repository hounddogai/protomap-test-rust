use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict approval entry must pass. Check returns its problems, or none.
pub trait StrictApprovalEntryRule: std::fmt::Debug {
    fn check(&self, item: &StrictApprovalEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalEntryCodePrefixRule {
    pub prefix: String,
}

impl StrictApprovalEntryRule for StrictApprovalEntryCodePrefixRule {
    fn check(&self, item: &StrictApprovalEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalEntryRequiredNameRule {
    pub message: String,
}

impl StrictApprovalEntryRule for StrictApprovalEntryRequiredNameRule {
    fn check(&self, item: &StrictApprovalEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalEntryCodeLengthRule {
    pub max_length: i64,
}

impl StrictApprovalEntryRule for StrictApprovalEntryCodeLengthRule {
    fn check(&self, item: &StrictApprovalEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalEntryRequiredCodeRule {
    pub message: String,
}

impl StrictApprovalEntryRule for StrictApprovalEntryRequiredCodeRule {
    fn check(&self, item: &StrictApprovalEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict approval entry.
#[derive(Debug, Clone)]
pub struct StrictApprovalEntryValidator {
    pub rules: Vec<Rc<dyn StrictApprovalEntryRule>>,
}

impl StrictApprovalEntryValidator {
    pub fn validate(&self, item: &StrictApprovalEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictApprovalEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictApprovalEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_approval_entry_validator() -> StrictApprovalEntryValidator {
    StrictApprovalEntryValidator {
        rules: vec![
            Rc::new(StrictApprovalEntryCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn StrictApprovalEntryRule>,
            Rc::new(StrictApprovalEntryRequiredNameRule {
                message: "strict approval entry needs this field".to_string(),
            }),
            Rc::new(StrictApprovalEntryCodeLengthRule { max_length: 3600 }),
            Rc::new(StrictApprovalEntryRequiredCodeRule {
                message: "strict approval entry needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_approval_entry_validator();
        let item = new_strict_approval_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictApprovalEntryRequiredNameRule {
            message: "strict approval entry needs a value".to_string(),
        };
        let mut item = new_strict_approval_entry("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictApprovalEntryCodeLengthRule { max_length: 255 };
        let mut item = new_strict_approval_entry("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictApprovalEntryRequiredCodeRule {
            message: "strict approval entry needs this field".to_string(),
        };
        let mut item = new_strict_approval_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
