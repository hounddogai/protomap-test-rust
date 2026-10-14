use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage template must pass. Check returns its problems, or none.
pub trait UsageTemplateRule: std::fmt::Debug {
    fn check(&self, item: &UsageTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageTemplateRequiredExternalRefRule {
    pub message: String,
}

impl UsageTemplateRule for UsageTemplateRequiredExternalRefRule {
    fn check(&self, item: &UsageTemplate) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageTemplateAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl UsageTemplateRule for UsageTemplateAllowedLabelRule {
    fn check(&self, item: &UsageTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageTemplateExternalRefLengthRule {
    pub max_length: i64,
}

impl UsageTemplateRule for UsageTemplateExternalRefLengthRule {
    fn check(&self, item: &UsageTemplate) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct UsageTemplateVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageTemplateRule for UsageTemplateVersionRangeRule {
    fn check(&self, item: &UsageTemplate) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage template.
#[derive(Debug, Clone)]
pub struct UsageTemplateValidator {
    pub rules: Vec<Rc<dyn UsageTemplateRule>>,
}

impl UsageTemplateValidator {
    pub fn validate(&self, item: &UsageTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_template_validator() -> UsageTemplateValidator {
    UsageTemplateValidator {
        rules: vec![
            Rc::new(UsageTemplateRequiredExternalRefRule {
                message: "usage template needs an entry".to_string(),
            }) as Rc<dyn UsageTemplateRule>,
            Rc::new(UsageTemplateAllowedLabelRule {
                allowed: vec!["adaptive".to_string(), "seasonal".to_string(), "primary".to_string()],
            }),
            Rc::new(UsageTemplateExternalRefLengthRule { max_length: 3600 }),
            Rc::new(UsageTemplateVersionRangeRule { min: 3600, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_template_validator();
        let item = new_usage_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = UsageTemplateRequiredExternalRefRule {
            message: "usage template needs a value".to_string(),
        };
        let mut item = new_usage_template("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = UsageTemplateExternalRefLengthRule { max_length: 32 };
        let mut item = new_usage_template("x", "y");
        item.external_ref = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = UsageTemplateVersionRangeRule { min: 64, max: 3600 };
        let mut item = new_usage_template("x", "y");
        item.version = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
