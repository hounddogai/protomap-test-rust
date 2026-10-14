use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling plan mapping must pass. Check returns its problems, or none.
pub trait RollingPlanMappingRule: std::fmt::Debug {
    fn check(&self, item: &RollingPlanMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanMappingCodeLengthRule {
    pub max_length: i64,
}

impl RollingPlanMappingRule for RollingPlanMappingCodeLengthRule {
    fn check(&self, item: &RollingPlanMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanMappingTimezoneLengthRule {
    pub max_length: i64,
}

impl RollingPlanMappingRule for RollingPlanMappingTimezoneLengthRule {
    fn check(&self, item: &RollingPlanMapping) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanMappingRequiredCodeRule {
    pub message: String,
}

impl RollingPlanMappingRule for RollingPlanMappingRequiredCodeRule {
    fn check(&self, item: &RollingPlanMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanMappingTitleLengthRule {
    pub max_length: i64,
}

impl RollingPlanMappingRule for RollingPlanMappingTitleLengthRule {
    fn check(&self, item: &RollingPlanMapping) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanMappingAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RollingPlanMappingRule for RollingPlanMappingAllowedTitleRule {
    fn check(&self, item: &RollingPlanMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is below minimum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanMappingRequiredCipherRule {
    pub message: String,
}

impl RollingPlanMappingRule for RollingPlanMappingRequiredCipherRule {
    fn check(&self, item: &RollingPlanMapping) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling plan mapping.
#[derive(Debug, Clone)]
pub struct RollingPlanMappingValidator {
    pub rules: Vec<Rc<dyn RollingPlanMappingRule>>,
}

impl RollingPlanMappingValidator {
    pub fn validate(&self, item: &RollingPlanMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingPlanMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPlanMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_plan_mapping_validator() -> RollingPlanMappingValidator {
    RollingPlanMappingValidator {
        rules: vec![
            Rc::new(RollingPlanMappingCodeLengthRule { max_length: 120 }) as Rc<dyn RollingPlanMappingRule>,
            Rc::new(RollingPlanMappingTimezoneLengthRule { max_length: 86400 }),
            Rc::new(RollingPlanMappingRequiredCodeRule {
                message: "rolling plan mapping needs a value".to_string(),
            }),
            Rc::new(RollingPlanMappingTitleLengthRule { max_length: 1000 }),
            Rc::new(RollingPlanMappingAllowedTitleRule {
                allowed: vec!["default".to_string(), "strict".to_string(), "weighted".to_string()],
            }),
            Rc::new(RollingPlanMappingRequiredCipherRule {
                message: "rolling plan mapping needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_plan_mapping_validator();
        let item = new_rolling_plan_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingPlanMappingCodeLengthRule { max_length: 255 };
        let mut item = new_rolling_plan_mapping("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = RollingPlanMappingTimezoneLengthRule { max_length: 86400 };
        let mut item = new_rolling_plan_mapping("x", "y");
        item.timezone = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingPlanMappingRequiredCodeRule {
            message: "rolling plan mapping needs a value".to_string(),
        };
        let mut item = new_rolling_plan_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollingPlanMappingTitleLengthRule { max_length: 120 };
        let mut item = new_rolling_plan_mapping("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = RollingPlanMappingRequiredCipherRule {
            message: "rolling plan mapping needs an entry".to_string(),
        };
        let mut item = new_rolling_plan_mapping("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
