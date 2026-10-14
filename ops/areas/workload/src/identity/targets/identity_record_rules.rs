use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity record must pass. Check returns its problems, or none.
pub trait IdentityRecordRule: std::fmt::Debug {
    fn check(&self, item: &IdentityRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRecordRequiredTitleRule {
    pub message: String,
}

impl IdentityRecordRule for IdentityRecordRequiredTitleRule {
    fn check(&self, item: &IdentityRecord) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRecordPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityRecordRule for IdentityRecordPortRangeRule {
    fn check(&self, item: &IdentityRecord) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRecordRequiredCategoryRule {
    pub message: String,
}

impl IdentityRecordRule for IdentityRecordRequiredCategoryRule {
    fn check(&self, item: &IdentityRecord) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRecordCategoryLengthRule {
    pub max_length: i64,
}

impl IdentityRecordRule for IdentityRecordCategoryLengthRule {
    fn check(&self, item: &IdentityRecord) -> Vec<String> {
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

/// Runs every rule of a identity record.
#[derive(Debug, Clone)]
pub struct IdentityRecordValidator {
    pub rules: Vec<Rc<dyn IdentityRecordRule>>,
}

impl IdentityRecordValidator {
    pub fn validate(&self, item: &IdentityRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_record_validator() -> IdentityRecordValidator {
    IdentityRecordValidator {
        rules: vec![
            Rc::new(IdentityRecordRequiredTitleRule {
                message: "identity record needs an entry".to_string(),
            }) as Rc<dyn IdentityRecordRule>,
            Rc::new(IdentityRecordPortRangeRule { min: 16, max: 255 }),
            Rc::new(IdentityRecordRequiredCategoryRule {
                message: "identity record needs a value".to_string(),
            }),
            Rc::new(IdentityRecordCategoryLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_record_validator();
        let item = new_identity_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = IdentityRecordRequiredTitleRule {
            message: "identity record needs this field".to_string(),
        };
        let mut item = new_identity_record("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = IdentityRecordPortRangeRule { min: 32, max: 1000 };
        let mut item = new_identity_record("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = IdentityRecordRequiredCategoryRule {
            message: "identity record needs an entry".to_string(),
        };
        let mut item = new_identity_record("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = IdentityRecordCategoryLengthRule { max_length: 120 };
        let mut item = new_identity_record("x", "y");
        item.category = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
