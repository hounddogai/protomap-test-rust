use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy binding template must pass. Check returns its problems, or none.
pub trait LegacyBindingTemplateRule: std::fmt::Debug {
    fn check(&self, item: &LegacyBindingTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingTemplateQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyBindingTemplateRule for LegacyBindingTemplateQuotaUnitsRangeRule {
    fn check(&self, item: &LegacyBindingTemplate) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingTemplateAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LegacyBindingTemplateRule for LegacyBindingTemplateAllowedTitleRule {
    fn check(&self, item: &LegacyBindingTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is unknown", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingTemplateCodeLengthRule {
    pub max_length: i64,
}

impl LegacyBindingTemplateRule for LegacyBindingTemplateCodeLengthRule {
    fn check(&self, item: &LegacyBindingTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingTemplateTitleLengthRule {
    pub max_length: i64,
}

impl LegacyBindingTemplateRule for LegacyBindingTemplateTitleLengthRule {
    fn check(&self, item: &LegacyBindingTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingTemplateRequiredIssuerRule {
    pub message: String,
}

impl LegacyBindingTemplateRule for LegacyBindingTemplateRequiredIssuerRule {
    fn check(&self, item: &LegacyBindingTemplate) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy binding template.
#[derive(Debug, Clone)]
pub struct LegacyBindingTemplateValidator {
    pub rules: Vec<Rc<dyn LegacyBindingTemplateRule>>,
}

impl LegacyBindingTemplateValidator {
    pub fn validate(&self, item: &LegacyBindingTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyBindingTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyBindingTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_binding_template_validator() -> LegacyBindingTemplateValidator {
    LegacyBindingTemplateValidator {
        rules: vec![
            Rc::new(LegacyBindingTemplateQuotaUnitsRangeRule { min: 16, max: 64 }) as Rc<dyn LegacyBindingTemplateRule>,
            Rc::new(LegacyBindingTemplateAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(LegacyBindingTemplateCodeLengthRule { max_length: 64 }),
            Rc::new(LegacyBindingTemplateTitleLengthRule { max_length: 86400 }),
            Rc::new(LegacyBindingTemplateRequiredIssuerRule {
                message: "legacy binding template needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_binding_template_validator();
        let item = new_legacy_binding_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = LegacyBindingTemplateQuotaUnitsRangeRule { min: 16, max: 3600 };
        let mut item = new_legacy_binding_template("x", "y");
        item.quota_units = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyBindingTemplateCodeLengthRule { max_length: 64 };
        let mut item = new_legacy_binding_template("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyBindingTemplateTitleLengthRule { max_length: 120 };
        let mut item = new_legacy_binding_template("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = LegacyBindingTemplateRequiredIssuerRule {
            message: "legacy binding template needs a value".to_string(),
        };
        let mut item = new_legacy_binding_template("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
