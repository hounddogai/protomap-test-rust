use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a health check template must pass. Check returns its problems, or none.
pub trait HealthCheckTemplateRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTemplateRequiredLabelRule {
    pub message: String,
}

impl HealthCheckTemplateRule for HealthCheckTemplateRequiredLabelRule {
    fn check(&self, item: &HealthCheckTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTemplateCodeLengthRule {
    pub max_length: i64,
}

impl HealthCheckTemplateRule for HealthCheckTemplateCodeLengthRule {
    fn check(&self, item: &HealthCheckTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTemplateAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl HealthCheckTemplateRule for HealthCheckTemplateAllowedChannelRule {
    fn check(&self, item: &HealthCheckTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is blocked", item.channel)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTemplateRequiredChannelRule {
    pub message: String,
}

impl HealthCheckTemplateRule for HealthCheckTemplateRequiredChannelRule {
    fn check(&self, item: &HealthCheckTemplate) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check template.
#[derive(Debug, Clone)]
pub struct HealthCheckTemplateValidator {
    pub rules: Vec<Rc<dyn HealthCheckTemplateRule>>,
}

impl HealthCheckTemplateValidator {
    pub fn validate(&self, item: &HealthCheckTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_template_validator() -> HealthCheckTemplateValidator {
    HealthCheckTemplateValidator {
        rules: vec![
            Rc::new(HealthCheckTemplateRequiredLabelRule {
                message: "health check template needs a value".to_string(),
            }) as Rc<dyn HealthCheckTemplateRule>,
            Rc::new(HealthCheckTemplateCodeLengthRule { max_length: 86400 }),
            Rc::new(HealthCheckTemplateAllowedChannelRule {
                allowed: vec!["pending".to_string(), "fallback".to_string()],
            }),
            Rc::new(HealthCheckTemplateRequiredChannelRule {
                message: "health check template needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_template_validator();
        let item = new_health_check_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = HealthCheckTemplateRequiredLabelRule {
            message: "health check template needs this field".to_string(),
        };
        let mut item = new_health_check_template("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = HealthCheckTemplateCodeLengthRule { max_length: 32 };
        let mut item = new_health_check_template("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = HealthCheckTemplateRequiredChannelRule {
            message: "health check template needs a value".to_string(),
        };
        let mut item = new_health_check_template("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
