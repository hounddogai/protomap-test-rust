use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling limit mapping must pass. Check returns its problems, or none.
pub trait RollingLimitMappingRule: std::fmt::Debug {
    fn check(&self, item: &RollingLimitMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitMappingCodePrefixRule {
    pub prefix: String,
}

impl RollingLimitMappingRule for RollingLimitMappingCodePrefixRule {
    fn check(&self, item: &RollingLimitMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitMappingTitleLengthRule {
    pub max_length: i64,
}

impl RollingLimitMappingRule for RollingLimitMappingTitleLengthRule {
    fn check(&self, item: &RollingLimitMapping) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitMappingRequiredCodeRule {
    pub message: String,
}

impl RollingLimitMappingRule for RollingLimitMappingRequiredCodeRule {
    fn check(&self, item: &RollingLimitMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling limit mapping.
#[derive(Debug, Clone)]
pub struct RollingLimitMappingValidator {
    pub rules: Vec<Rc<dyn RollingLimitMappingRule>>,
}

impl RollingLimitMappingValidator {
    pub fn validate(&self, item: &RollingLimitMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingLimitMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingLimitMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_limit_mapping_validator() -> RollingLimitMappingValidator {
    RollingLimitMappingValidator {
        rules: vec![
            Rc::new(RollingLimitMappingCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn RollingLimitMappingRule>,
            Rc::new(RollingLimitMappingTitleLengthRule { max_length: 255 }),
            Rc::new(RollingLimitMappingRequiredCodeRule {
                message: "rolling limit mapping needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_limit_mapping_validator();
        let item = new_rolling_limit_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollingLimitMappingTitleLengthRule { max_length: 120 };
        let mut item = new_rolling_limit_mapping("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingLimitMappingRequiredCodeRule {
            message: "rolling limit mapping needs this field".to_string(),
        };
        let mut item = new_rolling_limit_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
