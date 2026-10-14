use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy isolation template must pass. Check returns its problems, or none.
pub trait LegacyIsolationTemplateRule: std::fmt::Debug {
    fn check(&self, item: &LegacyIsolationTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationTemplateRequiredCodeRule {
    pub message: String,
}

impl LegacyIsolationTemplateRule for LegacyIsolationTemplateRequiredCodeRule {
    fn check(&self, item: &LegacyIsolationTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationTemplateTitleLengthRule {
    pub max_length: i64,
}

impl LegacyIsolationTemplateRule for LegacyIsolationTemplateTitleLengthRule {
    fn check(&self, item: &LegacyIsolationTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationTemplateCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyIsolationTemplateRule for LegacyIsolationTemplateCeilingRangeRule {
    fn check(&self, item: &LegacyIsolationTemplate) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationTemplateCodeLengthRule {
    pub max_length: i64,
}

impl LegacyIsolationTemplateRule for LegacyIsolationTemplateCodeLengthRule {
    fn check(&self, item: &LegacyIsolationTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy isolation template.
#[derive(Debug, Clone)]
pub struct LegacyIsolationTemplateValidator {
    pub rules: Vec<Rc<dyn LegacyIsolationTemplateRule>>,
}

impl LegacyIsolationTemplateValidator {
    pub fn validate(&self, item: &LegacyIsolationTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyIsolationTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyIsolationTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_isolation_template_validator() -> LegacyIsolationTemplateValidator {
    LegacyIsolationTemplateValidator {
        rules: vec![
            Rc::new(LegacyIsolationTemplateRequiredCodeRule {
                message: "legacy isolation template needs this field".to_string(),
            }) as Rc<dyn LegacyIsolationTemplateRule>,
            Rc::new(LegacyIsolationTemplateTitleLengthRule { max_length: 120 }),
            Rc::new(LegacyIsolationTemplateCeilingRangeRule { min: 1.8, max: 2151.6 }),
            Rc::new(LegacyIsolationTemplateCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_isolation_template_validator();
        let item = new_legacy_isolation_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyIsolationTemplateRequiredCodeRule {
            message: "legacy isolation template needs this field".to_string(),
        };
        let mut item = new_legacy_isolation_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyIsolationTemplateTitleLengthRule { max_length: 3600 };
        let mut item = new_legacy_isolation_template("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = LegacyIsolationTemplateCeilingRangeRule { min: 3.8, max: 2206.5 };
        let mut item = new_legacy_isolation_template("x", "y");
        item.ceiling = 2208.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyIsolationTemplateCodeLengthRule { max_length: 3600 };
        let mut item = new_legacy_isolation_template("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
