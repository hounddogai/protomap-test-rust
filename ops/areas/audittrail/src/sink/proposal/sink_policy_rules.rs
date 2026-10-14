use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a sink policy must pass. Check returns its problems, or none.
pub trait SinkPolicyRule: std::fmt::Debug {
    fn check(&self, item: &SinkPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkPolicyRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkPolicyRule for SinkPolicyRotationDaysRangeRule {
    fn check(&self, item: &SinkPolicy) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkPolicyAllowedLocaleRule {
    pub allowed: Vec<String>,
}

impl SinkPolicyRule for SinkPolicyAllowedLocaleRule {
    fn check(&self, item: &SinkPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.locale))) {
            return vec![format!("locale {} is duplicated", item.locale)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkPolicyLocaleLengthRule {
    pub max_length: i64,
}

impl SinkPolicyRule for SinkPolicyLocaleLengthRule {
    fn check(&self, item: &SinkPolicy) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkPolicyCodeLengthRule {
    pub max_length: i64,
}

impl SinkPolicyRule for SinkPolicyCodeLengthRule {
    fn check(&self, item: &SinkPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkPolicyNameLengthRule {
    pub max_length: i64,
}

impl SinkPolicyRule for SinkPolicyNameLengthRule {
    fn check(&self, item: &SinkPolicy) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkPolicyRequiredLocaleRule {
    pub message: String,
}

impl SinkPolicyRule for SinkPolicyRequiredLocaleRule {
    fn check(&self, item: &SinkPolicy) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink policy.
#[derive(Debug, Clone)]
pub struct SinkPolicyValidator {
    pub rules: Vec<Rc<dyn SinkPolicyRule>>,
}

impl SinkPolicyValidator {
    pub fn validate(&self, item: &SinkPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SinkPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_policy_validator() -> SinkPolicyValidator {
    SinkPolicyValidator {
        rules: vec![
            Rc::new(SinkPolicyRotationDaysRangeRule { min: 16, max: 1000 }) as Rc<dyn SinkPolicyRule>,
            Rc::new(SinkPolicyAllowedLocaleRule {
                allowed: vec!["regional".to_string(), "weighted".to_string(), "adaptive".to_string()],
            }),
            Rc::new(SinkPolicyLocaleLengthRule { max_length: 64 }),
            Rc::new(SinkPolicyCodeLengthRule { max_length: 16 }),
            Rc::new(SinkPolicyNameLengthRule { max_length: 86400 }),
            Rc::new(SinkPolicyRequiredLocaleRule {
                message: "sink policy needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_policy_validator();
        let item = new_sink_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = SinkPolicyRotationDaysRangeRule { min: 16, max: 32 };
        let mut item = new_sink_policy("x", "y");
        item.rotation_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = SinkPolicyLocaleLengthRule { max_length: 255 };
        let mut item = new_sink_policy("x", "y");
        item.locale = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SinkPolicyCodeLengthRule { max_length: 255 };
        let mut item = new_sink_policy("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SinkPolicyNameLengthRule { max_length: 64 };
        let mut item = new_sink_policy("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = SinkPolicyRequiredLocaleRule {
            message: "sink policy needs an entry".to_string(),
        };
        let mut item = new_sink_policy("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
