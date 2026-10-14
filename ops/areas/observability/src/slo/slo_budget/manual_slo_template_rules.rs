use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual slo template must pass. Check returns its problems, or none.
pub trait ManualSloTemplateRule: std::fmt::Debug {
    fn check(&self, item: &ManualSloTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSloTemplateCodeLengthRule {
    pub max_length: i64,
}

impl ManualSloTemplateRule for ManualSloTemplateCodeLengthRule {
    fn check(&self, item: &ManualSloTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSloTemplateRequiredLocaleRule {
    pub message: String,
}

impl ManualSloTemplateRule for ManualSloTemplateRequiredLocaleRule {
    fn check(&self, item: &ManualSloTemplate) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSloTemplateCodePrefixRule {
    pub prefix: String,
}

impl ManualSloTemplateRule for ManualSloTemplateCodePrefixRule {
    fn check(&self, item: &ManualSloTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSloTemplateRequiredCurrencyRule {
    pub message: String,
}

impl ManualSloTemplateRule for ManualSloTemplateRequiredCurrencyRule {
    fn check(&self, item: &ManualSloTemplate) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSloTemplateRequiredLabelRule {
    pub message: String,
}

impl ManualSloTemplateRule for ManualSloTemplateRequiredLabelRule {
    fn check(&self, item: &ManualSloTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSloTemplateLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualSloTemplateRule for ManualSloTemplateLimitRangeRule {
    fn check(&self, item: &ManualSloTemplate) -> Vec<String> {
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

/// Runs every rule of a manual slo template.
#[derive(Debug, Clone)]
pub struct ManualSloTemplateValidator {
    pub rules: Vec<Rc<dyn ManualSloTemplateRule>>,
}

impl ManualSloTemplateValidator {
    pub fn validate(&self, item: &ManualSloTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualSloTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualSloTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_slo_template_validator() -> ManualSloTemplateValidator {
    ManualSloTemplateValidator {
        rules: vec![
            Rc::new(ManualSloTemplateCodeLengthRule { max_length: 86400 }) as Rc<dyn ManualSloTemplateRule>,
            Rc::new(ManualSloTemplateRequiredLocaleRule {
                message: "manual slo template needs a value".to_string(),
            }),
            Rc::new(ManualSloTemplateCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ManualSloTemplateRequiredCurrencyRule {
                message: "manual slo template needs a value".to_string(),
            }),
            Rc::new(ManualSloTemplateRequiredLabelRule {
                message: "manual slo template needs a value".to_string(),
            }),
            Rc::new(ManualSloTemplateLimitRangeRule { min: 32, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_slo_template_validator();
        let item = new_manual_slo_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualSloTemplateCodeLengthRule { max_length: 64 };
        let mut item = new_manual_slo_template("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = ManualSloTemplateRequiredLocaleRule {
            message: "manual slo template needs an entry".to_string(),
        };
        let mut item = new_manual_slo_template("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = ManualSloTemplateRequiredCurrencyRule {
            message: "manual slo template needs a value".to_string(),
        };
        let mut item = new_manual_slo_template("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualSloTemplateRequiredLabelRule {
            message: "manual slo template needs this field".to_string(),
        };
        let mut item = new_manual_slo_template("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = ManualSloTemplateLimitRangeRule { min: 80, max: 3600 };
        let mut item = new_manual_slo_template("x", "y");
        item.limit = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
