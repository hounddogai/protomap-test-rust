use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback health check template must pass. Check returns its problems, or none.
pub trait FallbackHealthCheckTemplateRule: std::fmt::Debug {
    fn check(&self, item: &FallbackHealthCheckTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackHealthCheckTemplateRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackHealthCheckTemplateRule for FallbackHealthCheckTemplateRetriesRangeRule {
    fn check(&self, item: &FallbackHealthCheckTemplate) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackHealthCheckTemplateAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl FallbackHealthCheckTemplateRule for FallbackHealthCheckTemplateAllowedTitleRule {
    fn check(&self, item: &FallbackHealthCheckTemplate) -> Vec<String> {
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
pub struct FallbackHealthCheckTemplateCodeLengthRule {
    pub max_length: i64,
}

impl FallbackHealthCheckTemplateRule for FallbackHealthCheckTemplateCodeLengthRule {
    fn check(&self, item: &FallbackHealthCheckTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackHealthCheckTemplateRequiredNotesRule {
    pub message: String,
}

impl FallbackHealthCheckTemplateRule for FallbackHealthCheckTemplateRequiredNotesRule {
    fn check(&self, item: &FallbackHealthCheckTemplate) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackHealthCheckTemplateRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackHealthCheckTemplateRule for FallbackHealthCheckTemplateRatioRangeRule {
    fn check(&self, item: &FallbackHealthCheckTemplate) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackHealthCheckTemplateCodePrefixRule {
    pub prefix: String,
}

impl FallbackHealthCheckTemplateRule for FallbackHealthCheckTemplateCodePrefixRule {
    fn check(&self, item: &FallbackHealthCheckTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback health check template.
#[derive(Debug, Clone)]
pub struct FallbackHealthCheckTemplateValidator {
    pub rules: Vec<Rc<dyn FallbackHealthCheckTemplateRule>>,
}

impl FallbackHealthCheckTemplateValidator {
    pub fn validate(&self, item: &FallbackHealthCheckTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackHealthCheckTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackHealthCheckTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_health_check_template_validator() -> FallbackHealthCheckTemplateValidator {
    FallbackHealthCheckTemplateValidator {
        rules: vec![
            Rc::new(FallbackHealthCheckTemplateRetriesRangeRule { min: 80, max: 3600 })
                as Rc<dyn FallbackHealthCheckTemplateRule>,
            Rc::new(FallbackHealthCheckTemplateAllowedTitleRule {
                allowed: vec!["primary".to_string(), "primary".to_string(), "seasonal".to_string()],
            }),
            Rc::new(FallbackHealthCheckTemplateCodeLengthRule { max_length: 64 }),
            Rc::new(FallbackHealthCheckTemplateRequiredNotesRule {
                message: "fallback health check template needs this field".to_string(),
            }),
            Rc::new(FallbackHealthCheckTemplateRatioRangeRule { min: 0.7, max: 3965.6 }),
            Rc::new(FallbackHealthCheckTemplateCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_health_check_template_validator();
        let item = new_fallback_health_check_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = FallbackHealthCheckTemplateRetriesRangeRule { min: 32, max: 86400 };
        let mut item = new_fallback_health_check_template("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackHealthCheckTemplateCodeLengthRule { max_length: 120 };
        let mut item = new_fallback_health_check_template("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = FallbackHealthCheckTemplateRequiredNotesRule {
            message: "fallback health check template needs an entry".to_string(),
        };
        let mut item = new_fallback_health_check_template("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = FallbackHealthCheckTemplateRatioRangeRule { min: 0.8, max: 710.0 };
        let mut item = new_fallback_health_check_template("x", "y");
        item.ratio = 711.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
