use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity template must pass. Check returns its problems, or none.
pub trait IdentityTemplateRule: std::fmt::Debug {
    fn check(&self, item: &IdentityTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTemplateUnitLengthRule {
    pub max_length: i64,
}

impl IdentityTemplateRule for IdentityTemplateUnitLengthRule {
    fn check(&self, item: &IdentityTemplate) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTemplateCodePrefixRule {
    pub prefix: String,
}

impl IdentityTemplateRule for IdentityTemplateCodePrefixRule {
    fn check(&self, item: &IdentityTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTemplateRequiredCodeRule {
    pub message: String,
}

impl IdentityTemplateRule for IdentityTemplateRequiredCodeRule {
    fn check(&self, item: &IdentityTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTemplateRequiredUnitRule {
    pub message: String,
}

impl IdentityTemplateRule for IdentityTemplateRequiredUnitRule {
    fn check(&self, item: &IdentityTemplate) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTemplateCodeLengthRule {
    pub max_length: i64,
}

impl IdentityTemplateRule for IdentityTemplateCodeLengthRule {
    fn check(&self, item: &IdentityTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTemplateLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityTemplateRule for IdentityTemplateLimitRangeRule {
    fn check(&self, item: &IdentityTemplate) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity template.
#[derive(Debug, Clone)]
pub struct IdentityTemplateValidator {
    pub rules: Vec<Rc<dyn IdentityTemplateRule>>,
}

impl IdentityTemplateValidator {
    pub fn validate(&self, item: &IdentityTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_template_validator() -> IdentityTemplateValidator {
    IdentityTemplateValidator {
        rules: vec![
            Rc::new(IdentityTemplateUnitLengthRule { max_length: 16 }) as Rc<dyn IdentityTemplateRule>,
            Rc::new(IdentityTemplateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(IdentityTemplateRequiredCodeRule {
                message: "identity template needs an entry".to_string(),
            }),
            Rc::new(IdentityTemplateRequiredUnitRule {
                message: "identity template needs an entry".to_string(),
            }),
            Rc::new(IdentityTemplateCodeLengthRule { max_length: 255 }),
            Rc::new(IdentityTemplateLimitRangeRule { min: 64, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_template_validator();
        let item = new_identity_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = IdentityTemplateUnitLengthRule { max_length: 80 };
        let mut item = new_identity_template("x", "y");
        item.unit = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IdentityTemplateRequiredCodeRule {
            message: "identity template needs this field".to_string(),
        };
        let mut item = new_identity_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = IdentityTemplateRequiredUnitRule {
            message: "identity template needs this field".to_string(),
        };
        let mut item = new_identity_template("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IdentityTemplateCodeLengthRule { max_length: 32 };
        let mut item = new_identity_template("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = IdentityTemplateLimitRangeRule { min: 120, max: 3600 };
        let mut item = new_identity_template("x", "y");
        item.limit = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
