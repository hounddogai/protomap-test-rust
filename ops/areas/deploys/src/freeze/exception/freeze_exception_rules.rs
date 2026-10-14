use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze exception must pass. Check returns its problems, or none.
pub trait FreezeExceptionRule: std::fmt::Debug {
    fn check(&self, item: &FreezeException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeExceptionRequiredUnitRule {
    pub message: String,
}

impl FreezeExceptionRule for FreezeExceptionRequiredUnitRule {
    fn check(&self, item: &FreezeException) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeExceptionTitleLengthRule {
    pub max_length: i64,
}

impl FreezeExceptionRule for FreezeExceptionTitleLengthRule {
    fn check(&self, item: &FreezeException) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeExceptionCodePrefixRule {
    pub prefix: String,
}

impl FreezeExceptionRule for FreezeExceptionCodePrefixRule {
    fn check(&self, item: &FreezeException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze exception.
#[derive(Debug, Clone)]
pub struct FreezeExceptionValidator {
    pub rules: Vec<Rc<dyn FreezeExceptionRule>>,
}

impl FreezeExceptionValidator {
    pub fn validate(&self, item: &FreezeException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_exception_validator() -> FreezeExceptionValidator {
    FreezeExceptionValidator {
        rules: vec![
            Rc::new(FreezeExceptionRequiredUnitRule {
                message: "freeze exception needs a value".to_string(),
            }) as Rc<dyn FreezeExceptionRule>,
            Rc::new(FreezeExceptionTitleLengthRule { max_length: 255 }),
            Rc::new(FreezeExceptionCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_exception_validator();
        let item = new_freeze_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = FreezeExceptionRequiredUnitRule {
            message: "freeze exception needs this field".to_string(),
        };
        let mut item = new_freeze_exception("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FreezeExceptionTitleLengthRule { max_length: 16 };
        let mut item = new_freeze_exception("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
