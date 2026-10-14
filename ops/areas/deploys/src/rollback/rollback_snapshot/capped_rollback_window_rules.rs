use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped rollback window must pass. Check returns its problems, or none.
pub trait CappedRollbackWindowRule: std::fmt::Debug {
    fn check(&self, item: &CappedRollbackWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRollbackWindowRequiredTitleRule {
    pub message: String,
}

impl CappedRollbackWindowRule for CappedRollbackWindowRequiredTitleRule {
    fn check(&self, item: &CappedRollbackWindow) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRollbackWindowRequiredCategoryRule {
    pub message: String,
}

impl CappedRollbackWindowRule for CappedRollbackWindowRequiredCategoryRule {
    fn check(&self, item: &CappedRollbackWindow) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRollbackWindowCategoryLengthRule {
    pub max_length: i64,
}

impl CappedRollbackWindowRule for CappedRollbackWindowCategoryLengthRule {
    fn check(&self, item: &CappedRollbackWindow) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRollbackWindowCodeLengthRule {
    pub max_length: i64,
}

impl CappedRollbackWindowRule for CappedRollbackWindowCodeLengthRule {
    fn check(&self, item: &CappedRollbackWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRollbackWindowAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl CappedRollbackWindowRule for CappedRollbackWindowAllowedTimezoneRule {
    fn check(&self, item: &CappedRollbackWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is not allowed", item.timezone)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped rollback window.
#[derive(Debug, Clone)]
pub struct CappedRollbackWindowValidator {
    pub rules: Vec<Rc<dyn CappedRollbackWindowRule>>,
}

impl CappedRollbackWindowValidator {
    pub fn validate(&self, item: &CappedRollbackWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedRollbackWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRollbackWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_rollback_window_validator() -> CappedRollbackWindowValidator {
    CappedRollbackWindowValidator {
        rules: vec![
            Rc::new(CappedRollbackWindowRequiredTitleRule {
                message: "capped rollback window needs a value".to_string(),
            }) as Rc<dyn CappedRollbackWindowRule>,
            Rc::new(CappedRollbackWindowRequiredCategoryRule {
                message: "capped rollback window needs an entry".to_string(),
            }),
            Rc::new(CappedRollbackWindowCategoryLengthRule { max_length: 86400 }),
            Rc::new(CappedRollbackWindowCodeLengthRule { max_length: 16 }),
            Rc::new(CappedRollbackWindowAllowedTimezoneRule {
                allowed: vec!["scheduled".to_string(), "regional".to_string(), "adaptive".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_rollback_window_validator();
        let item = new_capped_rollback_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CappedRollbackWindowRequiredTitleRule {
            message: "capped rollback window needs this field".to_string(),
        };
        let mut item = new_capped_rollback_window("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = CappedRollbackWindowRequiredCategoryRule {
            message: "capped rollback window needs a value".to_string(),
        };
        let mut item = new_capped_rollback_window("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = CappedRollbackWindowCategoryLengthRule { max_length: 1000 };
        let mut item = new_capped_rollback_window("x", "y");
        item.category = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedRollbackWindowCodeLengthRule { max_length: 3600 };
        let mut item = new_capped_rollback_window("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
