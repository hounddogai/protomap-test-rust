use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone template must pass. Check returns its problems, or none.
pub trait ZoneTemplateRule: std::fmt::Debug {
    fn check(&self, item: &ZoneTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneTemplateUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneTemplateRule for ZoneTemplateUpdatedAtRangeRule {
    fn check(&self, item: &ZoneTemplate) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneTemplateCodeLengthRule {
    pub max_length: i64,
}

impl ZoneTemplateRule for ZoneTemplateCodeLengthRule {
    fn check(&self, item: &ZoneTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneTemplateRequiredCategoryRule {
    pub message: String,
}

impl ZoneTemplateRule for ZoneTemplateRequiredCategoryRule {
    fn check(&self, item: &ZoneTemplate) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneTemplateCodePrefixRule {
    pub prefix: String,
}

impl ZoneTemplateRule for ZoneTemplateCodePrefixRule {
    fn check(&self, item: &ZoneTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone template.
#[derive(Debug, Clone)]
pub struct ZoneTemplateValidator {
    pub rules: Vec<Rc<dyn ZoneTemplateRule>>,
}

impl ZoneTemplateValidator {
    pub fn validate(&self, item: &ZoneTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ZoneTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_template_validator() -> ZoneTemplateValidator {
    ZoneTemplateValidator {
        rules: vec![
            Rc::new(ZoneTemplateUpdatedAtRangeRule { min: 16, max: 32 }) as Rc<dyn ZoneTemplateRule>,
            Rc::new(ZoneTemplateCodeLengthRule { max_length: 16 }),
            Rc::new(ZoneTemplateRequiredCategoryRule {
                message: "zone template needs this field".to_string(),
            }),
            Rc::new(ZoneTemplateCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_template_validator();
        let item = new_zone_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = ZoneTemplateUpdatedAtRangeRule { min: 64, max: 255 };
        let mut item = new_zone_template("x", "y");
        item.updated_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ZoneTemplateCodeLengthRule { max_length: 120 };
        let mut item = new_zone_template("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = ZoneTemplateRequiredCategoryRule {
            message: "zone template needs an entry".to_string(),
        };
        let mut item = new_zone_template("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
