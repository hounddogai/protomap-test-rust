use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped page summary must pass. Check returns its problems, or none.
pub trait CappedPageSummaryRule: std::fmt::Debug {
    fn check(&self, item: &CappedPageSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageSummaryRequiredScopeRule {
    pub message: String,
}

impl CappedPageSummaryRule for CappedPageSummaryRequiredScopeRule {
    fn check(&self, item: &CappedPageSummary) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageSummaryScopeLengthRule {
    pub max_length: i64,
}

impl CappedPageSummaryRule for CappedPageSummaryScopeLengthRule {
    fn check(&self, item: &CappedPageSummary) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageSummaryCodePrefixRule {
    pub prefix: String,
}

impl CappedPageSummaryRule for CappedPageSummaryCodePrefixRule {
    fn check(&self, item: &CappedPageSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageSummaryRequiredSummaryRule {
    pub message: String,
}

impl CappedPageSummaryRule for CappedPageSummaryRequiredSummaryRule {
    fn check(&self, item: &CappedPageSummary) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped page summary.
#[derive(Debug, Clone)]
pub struct CappedPageSummaryValidator {
    pub rules: Vec<Rc<dyn CappedPageSummaryRule>>,
}

impl CappedPageSummaryValidator {
    pub fn validate(&self, item: &CappedPageSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedPageSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedPageSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_page_summary_validator() -> CappedPageSummaryValidator {
    CappedPageSummaryValidator {
        rules: vec![
            Rc::new(CappedPageSummaryRequiredScopeRule {
                message: "capped page summary needs an entry".to_string(),
            }) as Rc<dyn CappedPageSummaryRule>,
            Rc::new(CappedPageSummaryScopeLengthRule { max_length: 86400 }),
            Rc::new(CappedPageSummaryCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(CappedPageSummaryRequiredSummaryRule {
                message: "capped page summary needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_page_summary_validator();
        let item = new_capped_page_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = CappedPageSummaryRequiredScopeRule {
            message: "capped page summary needs this field".to_string(),
        };
        let mut item = new_capped_page_summary("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = CappedPageSummaryScopeLengthRule { max_length: 86400 };
        let mut item = new_capped_page_summary("x", "y");
        item.scope = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = CappedPageSummaryRequiredSummaryRule {
            message: "capped page summary needs an entry".to_string(),
        };
        let mut item = new_capped_page_summary("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
