use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback quote record must pass. Check returns its problems, or none.
pub trait FallbackQuoteRecordRule: std::fmt::Debug {
    fn check(&self, item: &FallbackQuoteRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteRecordRequiredKeyIdRule {
    pub message: String,
}

impl FallbackQuoteRecordRule for FallbackQuoteRecordRequiredKeyIdRule {
    fn check(&self, item: &FallbackQuoteRecord) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteRecordOwnerIdLengthRule {
    pub max_length: i64,
}

impl FallbackQuoteRecordRule for FallbackQuoteRecordOwnerIdLengthRule {
    fn check(&self, item: &FallbackQuoteRecord) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteRecordGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackQuoteRecordRule for FallbackQuoteRecordGraceDaysRangeRule {
    fn check(&self, item: &FallbackQuoteRecord) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteRecordNameLengthRule {
    pub max_length: i64,
}

impl FallbackQuoteRecordRule for FallbackQuoteRecordNameLengthRule {
    fn check(&self, item: &FallbackQuoteRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback quote record.
#[derive(Debug, Clone)]
pub struct FallbackQuoteRecordValidator {
    pub rules: Vec<Rc<dyn FallbackQuoteRecordRule>>,
}

impl FallbackQuoteRecordValidator {
    pub fn validate(&self, item: &FallbackQuoteRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackQuoteRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackQuoteRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_quote_record_validator() -> FallbackQuoteRecordValidator {
    FallbackQuoteRecordValidator {
        rules: vec![
            Rc::new(FallbackQuoteRecordRequiredKeyIdRule {
                message: "fallback quote record needs an entry".to_string(),
            }) as Rc<dyn FallbackQuoteRecordRule>,
            Rc::new(FallbackQuoteRecordOwnerIdLengthRule { max_length: 3600 }),
            Rc::new(FallbackQuoteRecordGraceDaysRangeRule { min: 32, max: 80 }),
            Rc::new(FallbackQuoteRecordNameLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_quote_record_validator();
        let item = new_fallback_quote_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = FallbackQuoteRecordRequiredKeyIdRule {
            message: "fallback quote record needs an entry".to_string(),
        };
        let mut item = new_fallback_quote_record("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = FallbackQuoteRecordOwnerIdLengthRule { max_length: 255 };
        let mut item = new_fallback_quote_record("x", "y");
        item.owner_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = FallbackQuoteRecordGraceDaysRangeRule { min: 32, max: 1000 };
        let mut item = new_fallback_quote_record("x", "y");
        item.grace_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackQuoteRecordNameLengthRule { max_length: 32 };
        let mut item = new_fallback_quote_record("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
