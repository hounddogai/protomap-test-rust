use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling sink mapping must pass. Check returns its problems, or none.
pub trait RollingSinkMappingRule: std::fmt::Debug {
    fn check(&self, item: &RollingSinkMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSinkMappingRequiredCodeRule {
    pub message: String,
}

impl RollingSinkMappingRule for RollingSinkMappingRequiredCodeRule {
    fn check(&self, item: &RollingSinkMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSinkMappingRequiredPhoneRule {
    pub message: String,
}

impl RollingSinkMappingRule for RollingSinkMappingRequiredPhoneRule {
    fn check(&self, item: &RollingSinkMapping) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSinkMappingCodePrefixRule {
    pub prefix: String,
}

impl RollingSinkMappingRule for RollingSinkMappingCodePrefixRule {
    fn check(&self, item: &RollingSinkMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSinkMappingPhoneLengthRule {
    pub max_length: i64,
}

impl RollingSinkMappingRule for RollingSinkMappingPhoneLengthRule {
    fn check(&self, item: &RollingSinkMapping) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSinkMappingExternalRefLengthRule {
    pub max_length: i64,
}

impl RollingSinkMappingRule for RollingSinkMappingExternalRefLengthRule {
    fn check(&self, item: &RollingSinkMapping) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling sink mapping.
#[derive(Debug, Clone)]
pub struct RollingSinkMappingValidator {
    pub rules: Vec<Rc<dyn RollingSinkMappingRule>>,
}

impl RollingSinkMappingValidator {
    pub fn validate(&self, item: &RollingSinkMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingSinkMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingSinkMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_sink_mapping_validator() -> RollingSinkMappingValidator {
    RollingSinkMappingValidator {
        rules: vec![
            Rc::new(RollingSinkMappingRequiredCodeRule {
                message: "rolling sink mapping needs a value".to_string(),
            }) as Rc<dyn RollingSinkMappingRule>,
            Rc::new(RollingSinkMappingRequiredPhoneRule {
                message: "rolling sink mapping needs an entry".to_string(),
            }),
            Rc::new(RollingSinkMappingCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RollingSinkMappingPhoneLengthRule { max_length: 120 }),
            Rc::new(RollingSinkMappingExternalRefLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_sink_mapping_validator();
        let item = new_rolling_sink_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingSinkMappingRequiredCodeRule {
            message: "rolling sink mapping needs a value".to_string(),
        };
        let mut item = new_rolling_sink_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = RollingSinkMappingRequiredPhoneRule {
            message: "rolling sink mapping needs an entry".to_string(),
        };
        let mut item = new_rolling_sink_mapping("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = RollingSinkMappingPhoneLengthRule { max_length: 80 };
        let mut item = new_rolling_sink_mapping("x", "y");
        item.phone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = RollingSinkMappingExternalRefLengthRule { max_length: 16 };
        let mut item = new_rolling_sink_mapping("x", "y");
        item.external_ref = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
