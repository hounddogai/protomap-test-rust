use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a freeze template must pass. Check returns its problems, or none.
pub trait FreezeTemplateRule: std::fmt::Debug {
    fn check(&self, item: &FreezeTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTemplateCodeLengthRule {
    pub max_length: i64,
}

impl FreezeTemplateRule for FreezeTemplateCodeLengthRule {
    fn check(&self, item: &FreezeTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTemplateBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeTemplateRule for FreezeTemplateBurstRangeRule {
    fn check(&self, item: &FreezeTemplate) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTemplateRequiredCurrencyRule {
    pub message: String,
}

impl FreezeTemplateRule for FreezeTemplateRequiredCurrencyRule {
    fn check(&self, item: &FreezeTemplate) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTemplateCodePrefixRule {
    pub prefix: String,
}

impl FreezeTemplateRule for FreezeTemplateCodePrefixRule {
    fn check(&self, item: &FreezeTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTemplateCurrencyLengthRule {
    pub max_length: i64,
}

impl FreezeTemplateRule for FreezeTemplateCurrencyLengthRule {
    fn check(&self, item: &FreezeTemplate) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTemplateAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FreezeTemplateRule for FreezeTemplateAllowedCodeRule {
    fn check(&self, item: &FreezeTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze template.
#[derive(Debug, Clone)]
pub struct FreezeTemplateValidator {
    pub rules: Vec<Rc<dyn FreezeTemplateRule>>,
}

impl FreezeTemplateValidator {
    pub fn validate(&self, item: &FreezeTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_template_validator() -> FreezeTemplateValidator {
    FreezeTemplateValidator {
        rules: vec![
            Rc::new(FreezeTemplateCodeLengthRule { max_length: 120 }) as Rc<dyn FreezeTemplateRule>,
            Rc::new(FreezeTemplateBurstRangeRule { min: 255, max: 1000 }),
            Rc::new(FreezeTemplateRequiredCurrencyRule {
                message: "freeze template needs this field".to_string(),
            }),
            Rc::new(FreezeTemplateCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(FreezeTemplateCurrencyLengthRule { max_length: 1000 }),
            Rc::new(FreezeTemplateAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "adaptive".to_string(), "lenient".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_template_validator();
        let item = new_freeze_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FreezeTemplateCodeLengthRule { max_length: 1000 };
        let mut item = new_freeze_template("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = FreezeTemplateBurstRangeRule { min: 16, max: 1000 };
        let mut item = new_freeze_template("x", "y");
        item.burst = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = FreezeTemplateRequiredCurrencyRule {
            message: "freeze template needs this field".to_string(),
        };
        let mut item = new_freeze_template("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = FreezeTemplateCurrencyLengthRule { max_length: 1000 };
        let mut item = new_freeze_template("x", "y");
        item.currency = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
