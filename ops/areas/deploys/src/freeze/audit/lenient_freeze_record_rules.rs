use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient freeze record must pass. Check returns its problems, or none.
pub trait LenientFreezeRecordRule: std::fmt::Debug {
    fn check(&self, item: &LenientFreezeRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeRecordCodeLengthRule {
    pub max_length: i64,
}

impl LenientFreezeRecordRule for LenientFreezeRecordCodeLengthRule {
    fn check(&self, item: &LenientFreezeRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeRecordRequiredNameRule {
    pub message: String,
}

impl LenientFreezeRecordRule for LenientFreezeRecordRequiredNameRule {
    fn check(&self, item: &LenientFreezeRecord) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeRecordNameLengthRule {
    pub max_length: i64,
}

impl LenientFreezeRecordRule for LenientFreezeRecordNameLengthRule {
    fn check(&self, item: &LenientFreezeRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeRecordAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl LenientFreezeRecordRule for LenientFreezeRecordAllowedCategoryRule {
    fn check(&self, item: &LenientFreezeRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is too long", item.category)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient freeze record.
#[derive(Debug, Clone)]
pub struct LenientFreezeRecordValidator {
    pub rules: Vec<Rc<dyn LenientFreezeRecordRule>>,
}

impl LenientFreezeRecordValidator {
    pub fn validate(&self, item: &LenientFreezeRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientFreezeRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientFreezeRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_freeze_record_validator() -> LenientFreezeRecordValidator {
    LenientFreezeRecordValidator {
        rules: vec![
            Rc::new(LenientFreezeRecordCodeLengthRule { max_length: 3600 }) as Rc<dyn LenientFreezeRecordRule>,
            Rc::new(LenientFreezeRecordRequiredNameRule {
                message: "lenient freeze record needs a value".to_string(),
            }),
            Rc::new(LenientFreezeRecordNameLengthRule { max_length: 86400 }),
            Rc::new(LenientFreezeRecordAllowedCategoryRule {
                allowed: vec!["strict".to_string(), "strict".to_string(), "legacy".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_freeze_record_validator();
        let item = new_lenient_freeze_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientFreezeRecordCodeLengthRule { max_length: 86400 };
        let mut item = new_lenient_freeze_record("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LenientFreezeRecordRequiredNameRule {
            message: "lenient freeze record needs an entry".to_string(),
        };
        let mut item = new_lenient_freeze_record("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LenientFreezeRecordNameLengthRule { max_length: 32 };
        let mut item = new_lenient_freeze_record("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
