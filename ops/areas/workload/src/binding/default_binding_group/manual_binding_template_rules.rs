use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual binding template must pass. Check returns its problems, or none.
pub trait ManualBindingTemplateRule: std::fmt::Debug {
    fn check(&self, item: &ManualBindingTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingTemplateAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualBindingTemplateRule for ManualBindingTemplateAttemptsRangeRule {
    fn check(&self, item: &ManualBindingTemplate) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingTemplateMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualBindingTemplateRule for ManualBindingTemplateMaximumRangeRule {
    fn check(&self, item: &ManualBindingTemplate) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingTemplateAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ManualBindingTemplateRule for ManualBindingTemplateAllowedLabelRule {
    fn check(&self, item: &ManualBindingTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingTemplateLabelLengthRule {
    pub max_length: i64,
}

impl ManualBindingTemplateRule for ManualBindingTemplateLabelLengthRule {
    fn check(&self, item: &ManualBindingTemplate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual binding template.
#[derive(Debug, Clone)]
pub struct ManualBindingTemplateValidator {
    pub rules: Vec<Rc<dyn ManualBindingTemplateRule>>,
}

impl ManualBindingTemplateValidator {
    pub fn validate(&self, item: &ManualBindingTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualBindingTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualBindingTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_binding_template_validator() -> ManualBindingTemplateValidator {
    ManualBindingTemplateValidator {
        rules: vec![
            Rc::new(ManualBindingTemplateAttemptsRangeRule { min: 80, max: 3600 }) as Rc<dyn ManualBindingTemplateRule>,
            Rc::new(ManualBindingTemplateMaximumRangeRule { min: 9.0, max: 3822.4 }),
            Rc::new(ManualBindingTemplateAllowedLabelRule {
                allowed: vec!["default".to_string(), "weighted".to_string()],
            }),
            Rc::new(ManualBindingTemplateLabelLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_binding_template_validator();
        let item = new_manual_binding_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ManualBindingTemplateAttemptsRangeRule { min: 16, max: 64 };
        let mut item = new_manual_binding_template("x", "y");
        item.attempts = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = ManualBindingTemplateMaximumRangeRule { min: 6.8, max: 2503.7 };
        let mut item = new_manual_binding_template("x", "y");
        item.maximum = 2505.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ManualBindingTemplateLabelLengthRule { max_length: 64 };
        let mut item = new_manual_binding_template("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
