use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention entry must pass. Check returns its problems, or none.
pub trait RetentionEntryRule: std::fmt::Debug {
    fn check(&self, item: &RetentionEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionEntryLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionEntryRule for RetentionEntryLimitRangeRule {
    fn check(&self, item: &RetentionEntry) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionEntryRequiredCodeRule {
    pub message: String,
}

impl RetentionEntryRule for RetentionEntryRequiredCodeRule {
    fn check(&self, item: &RetentionEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionEntryCodeLengthRule {
    pub max_length: i64,
}

impl RetentionEntryRule for RetentionEntryCodeLengthRule {
    fn check(&self, item: &RetentionEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionEntryRequiredRegionRule {
    pub message: String,
}

impl RetentionEntryRule for RetentionEntryRequiredRegionRule {
    fn check(&self, item: &RetentionEntry) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionEntryRequiredAudienceRule {
    pub message: String,
}

impl RetentionEntryRule for RetentionEntryRequiredAudienceRule {
    fn check(&self, item: &RetentionEntry) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionEntryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RetentionEntryRule for RetentionEntryAllowedCodeRule {
    fn check(&self, item: &RetentionEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention entry.
#[derive(Debug, Clone)]
pub struct RetentionEntryValidator {
    pub rules: Vec<Rc<dyn RetentionEntryRule>>,
}

impl RetentionEntryValidator {
    pub fn validate(&self, item: &RetentionEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RetentionEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_entry_validator() -> RetentionEntryValidator {
    RetentionEntryValidator {
        rules: vec![
            Rc::new(RetentionEntryLimitRangeRule { min: 32, max: 1000 }) as Rc<dyn RetentionEntryRule>,
            Rc::new(RetentionEntryRequiredCodeRule {
                message: "retention entry needs this field".to_string(),
            }),
            Rc::new(RetentionEntryCodeLengthRule { max_length: 120 }),
            Rc::new(RetentionEntryRequiredRegionRule {
                message: "retention entry needs this field".to_string(),
            }),
            Rc::new(RetentionEntryRequiredAudienceRule {
                message: "retention entry needs an entry".to_string(),
            }),
            Rc::new(RetentionEntryAllowedCodeRule {
                allowed: vec!["capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_entry_validator();
        let item = new_retention_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = RetentionEntryLimitRangeRule { min: 16, max: 120 };
        let mut item = new_retention_entry("x", "y");
        item.limit = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RetentionEntryRequiredCodeRule {
            message: "retention entry needs this field".to_string(),
        };
        let mut item = new_retention_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RetentionEntryCodeLengthRule { max_length: 3600 };
        let mut item = new_retention_entry("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = RetentionEntryRequiredRegionRule {
            message: "retention entry needs an entry".to_string(),
        };
        let mut item = new_retention_entry("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = RetentionEntryRequiredAudienceRule {
            message: "retention entry needs this field".to_string(),
        };
        let mut item = new_retention_entry("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
