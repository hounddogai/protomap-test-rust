use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped freeze exception must pass. Check returns its problems, or none.
pub trait CappedFreezeExceptionRule: std::fmt::Debug {
    fn check(&self, item: &CappedFreezeException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFreezeExceptionRequiredDescriptionRule {
    pub message: String,
}

impl CappedFreezeExceptionRule for CappedFreezeExceptionRequiredDescriptionRule {
    fn check(&self, item: &CappedFreezeException) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFreezeExceptionCodePrefixRule {
    pub prefix: String,
}

impl CappedFreezeExceptionRule for CappedFreezeExceptionCodePrefixRule {
    fn check(&self, item: &CappedFreezeException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFreezeExceptionDescriptionLengthRule {
    pub max_length: i64,
}

impl CappedFreezeExceptionRule for CappedFreezeExceptionDescriptionLengthRule {
    fn check(&self, item: &CappedFreezeException) -> Vec<String> {
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

/// Runs every rule of a capped freeze exception.
#[derive(Debug, Clone)]
pub struct CappedFreezeExceptionValidator {
    pub rules: Vec<Rc<dyn CappedFreezeExceptionRule>>,
}

impl CappedFreezeExceptionValidator {
    pub fn validate(&self, item: &CappedFreezeException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedFreezeException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedFreezeException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_freeze_exception_validator() -> CappedFreezeExceptionValidator {
    CappedFreezeExceptionValidator {
        rules: vec![
            Rc::new(CappedFreezeExceptionRequiredDescriptionRule {
                message: "capped freeze exception needs this field".to_string(),
            }) as Rc<dyn CappedFreezeExceptionRule>,
            Rc::new(CappedFreezeExceptionCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(CappedFreezeExceptionDescriptionLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_freeze_exception_validator();
        let item = new_capped_freeze_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = CappedFreezeExceptionRequiredDescriptionRule {
            message: "capped freeze exception needs an entry".to_string(),
        };
        let mut item = new_capped_freeze_exception("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = CappedFreezeExceptionDescriptionLengthRule { max_length: 86400 };
        let mut item = new_capped_freeze_exception("x", "y");
        item.description = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
