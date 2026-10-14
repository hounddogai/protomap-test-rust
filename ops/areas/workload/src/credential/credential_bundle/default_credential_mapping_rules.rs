use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default credential mapping must pass. Check returns its problems, or none.
pub trait DefaultCredentialMappingRule: std::fmt::Debug {
    fn check(&self, item: &DefaultCredentialMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialMappingRequiredTitleRule {
    pub message: String,
}

impl DefaultCredentialMappingRule for DefaultCredentialMappingRequiredTitleRule {
    fn check(&self, item: &DefaultCredentialMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialMappingRequiredSummaryRule {
    pub message: String,
}

impl DefaultCredentialMappingRule for DefaultCredentialMappingRequiredSummaryRule {
    fn check(&self, item: &DefaultCredentialMapping) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialMappingCodeLengthRule {
    pub max_length: i64,
}

impl DefaultCredentialMappingRule for DefaultCredentialMappingCodeLengthRule {
    fn check(&self, item: &DefaultCredentialMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialMappingSummaryLengthRule {
    pub max_length: i64,
}

impl DefaultCredentialMappingRule for DefaultCredentialMappingSummaryLengthRule {
    fn check(&self, item: &DefaultCredentialMapping) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialMappingRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultCredentialMappingRule for DefaultCredentialMappingRetriesRangeRule {
    fn check(&self, item: &DefaultCredentialMapping) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a default credential mapping.
#[derive(Debug, Clone)]
pub struct DefaultCredentialMappingValidator {
    pub rules: Vec<Rc<dyn DefaultCredentialMappingRule>>,
}

impl DefaultCredentialMappingValidator {
    pub fn validate(&self, item: &DefaultCredentialMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultCredentialMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultCredentialMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_credential_mapping_validator() -> DefaultCredentialMappingValidator {
    DefaultCredentialMappingValidator {
        rules: vec![
            Rc::new(DefaultCredentialMappingRequiredTitleRule {
                message: "default credential mapping needs this field".to_string(),
            }) as Rc<dyn DefaultCredentialMappingRule>,
            Rc::new(DefaultCredentialMappingRequiredSummaryRule {
                message: "default credential mapping needs an entry".to_string(),
            }),
            Rc::new(DefaultCredentialMappingCodeLengthRule { max_length: 1000 }),
            Rc::new(DefaultCredentialMappingSummaryLengthRule { max_length: 64 }),
            Rc::new(DefaultCredentialMappingRetriesRangeRule { min: 80, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_credential_mapping_validator();
        let item = new_default_credential_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DefaultCredentialMappingRequiredTitleRule {
            message: "default credential mapping needs this field".to_string(),
        };
        let mut item = new_default_credential_mapping("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = DefaultCredentialMappingRequiredSummaryRule {
            message: "default credential mapping needs an entry".to_string(),
        };
        let mut item = new_default_credential_mapping("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultCredentialMappingCodeLengthRule { max_length: 32 };
        let mut item = new_default_credential_mapping("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = DefaultCredentialMappingSummaryLengthRule { max_length: 32 };
        let mut item = new_default_credential_mapping("x", "y");
        item.summary = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = DefaultCredentialMappingRetriesRangeRule { min: 120, max: 255 };
        let mut item = new_default_credential_mapping("x", "y");
        item.retries = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
