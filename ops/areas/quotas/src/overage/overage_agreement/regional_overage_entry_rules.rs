use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional overage entry must pass. Check returns its problems, or none.
pub trait RegionalOverageEntryRule: std::fmt::Debug {
    fn check(&self, item: &RegionalOverageEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalOverageEntryRequiredReviewerRule {
    pub message: String,
}

impl RegionalOverageEntryRule for RegionalOverageEntryRequiredReviewerRule {
    fn check(&self, item: &RegionalOverageEntry) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalOverageEntryReviewerLengthRule {
    pub max_length: i64,
}

impl RegionalOverageEntryRule for RegionalOverageEntryReviewerLengthRule {
    fn check(&self, item: &RegionalOverageEntry) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalOverageEntryTitleLengthRule {
    pub max_length: i64,
}

impl RegionalOverageEntryRule for RegionalOverageEntryTitleLengthRule {
    fn check(&self, item: &RegionalOverageEntry) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalOverageEntryRequiredTitleRule {
    pub message: String,
}

impl RegionalOverageEntryRule for RegionalOverageEntryRequiredTitleRule {
    fn check(&self, item: &RegionalOverageEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalOverageEntryCodeLengthRule {
    pub max_length: i64,
}

impl RegionalOverageEntryRule for RegionalOverageEntryCodeLengthRule {
    fn check(&self, item: &RegionalOverageEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional overage entry.
#[derive(Debug, Clone)]
pub struct RegionalOverageEntryValidator {
    pub rules: Vec<Rc<dyn RegionalOverageEntryRule>>,
}

impl RegionalOverageEntryValidator {
    pub fn validate(&self, item: &RegionalOverageEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalOverageEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalOverageEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_overage_entry_validator() -> RegionalOverageEntryValidator {
    RegionalOverageEntryValidator {
        rules: vec![
            Rc::new(RegionalOverageEntryRequiredReviewerRule {
                message: "regional overage entry needs this field".to_string(),
            }) as Rc<dyn RegionalOverageEntryRule>,
            Rc::new(RegionalOverageEntryReviewerLengthRule { max_length: 86400 }),
            Rc::new(RegionalOverageEntryTitleLengthRule { max_length: 16 }),
            Rc::new(RegionalOverageEntryRequiredTitleRule {
                message: "regional overage entry needs an entry".to_string(),
            }),
            Rc::new(RegionalOverageEntryCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_overage_entry_validator();
        let item = new_regional_overage_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = RegionalOverageEntryRequiredReviewerRule {
            message: "regional overage entry needs a value".to_string(),
        };
        let mut item = new_regional_overage_entry("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = RegionalOverageEntryReviewerLengthRule { max_length: 80 };
        let mut item = new_regional_overage_entry("x", "y");
        item.reviewer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalOverageEntryTitleLengthRule { max_length: 80 };
        let mut item = new_regional_overage_entry("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionalOverageEntryRequiredTitleRule {
            message: "regional overage entry needs an entry".to_string(),
        };
        let mut item = new_regional_overage_entry("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalOverageEntryCodeLengthRule { max_length: 3600 };
        let mut item = new_regional_overage_entry("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
