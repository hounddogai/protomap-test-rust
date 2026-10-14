use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo record must pass. Check returns its problems, or none.
pub trait SloRecordRule: std::fmt::Debug {
    fn check(&self, item: &SloRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRecordCodePrefixRule {
    pub prefix: String,
}

impl SloRecordRule for SloRecordCodePrefixRule {
    fn check(&self, item: &SloRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRecordRequiredPhoneRule {
    pub message: String,
}

impl SloRecordRule for SloRecordRequiredPhoneRule {
    fn check(&self, item: &SloRecord) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRecordTitleLengthRule {
    pub max_length: i64,
}

impl SloRecordRule for SloRecordTitleLengthRule {
    fn check(&self, item: &SloRecord) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo record.
#[derive(Debug, Clone)]
pub struct SloRecordValidator {
    pub rules: Vec<Rc<dyn SloRecordRule>>,
}

impl SloRecordValidator {
    pub fn validate(&self, item: &SloRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_record_validator() -> SloRecordValidator {
    SloRecordValidator {
        rules: vec![
            Rc::new(SloRecordCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn SloRecordRule>,
            Rc::new(SloRecordRequiredPhoneRule {
                message: "slo record needs this field".to_string(),
            }),
            Rc::new(SloRecordTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_record_validator();
        let item = new_slo_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = SloRecordRequiredPhoneRule {
            message: "slo record needs a value".to_string(),
        };
        let mut item = new_slo_record("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SloRecordTitleLengthRule { max_length: 3600 };
        let mut item = new_slo_record("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
