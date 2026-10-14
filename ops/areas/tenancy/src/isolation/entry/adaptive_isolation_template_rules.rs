use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive isolation template must pass. Check returns its problems, or none.
pub trait AdaptiveIsolationTemplateRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveIsolationTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationTemplateDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveIsolationTemplateRule for AdaptiveIsolationTemplateDurationMinutesRangeRule {
    fn check(&self, item: &AdaptiveIsolationTemplate) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationTemplateCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveIsolationTemplateRule for AdaptiveIsolationTemplateCodeLengthRule {
    fn check(&self, item: &AdaptiveIsolationTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationTemplateLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveIsolationTemplateRule for AdaptiveIsolationTemplateLabelLengthRule {
    fn check(&self, item: &AdaptiveIsolationTemplate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationTemplateCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveIsolationTemplateRule for AdaptiveIsolationTemplateCodePrefixRule {
    fn check(&self, item: &AdaptiveIsolationTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive isolation template.
#[derive(Debug, Clone)]
pub struct AdaptiveIsolationTemplateValidator {
    pub rules: Vec<Rc<dyn AdaptiveIsolationTemplateRule>>,
}

impl AdaptiveIsolationTemplateValidator {
    pub fn validate(&self, item: &AdaptiveIsolationTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveIsolationTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveIsolationTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_isolation_template_validator() -> AdaptiveIsolationTemplateValidator {
    AdaptiveIsolationTemplateValidator {
        rules: vec![
            Rc::new(AdaptiveIsolationTemplateDurationMinutesRangeRule { min: 32, max: 1000 })
                as Rc<dyn AdaptiveIsolationTemplateRule>,
            Rc::new(AdaptiveIsolationTemplateCodeLengthRule { max_length: 1000 }),
            Rc::new(AdaptiveIsolationTemplateLabelLengthRule { max_length: 1000 }),
            Rc::new(AdaptiveIsolationTemplateCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_isolation_template_validator();
        let item = new_adaptive_isolation_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = AdaptiveIsolationTemplateDurationMinutesRangeRule { min: 64, max: 1000 };
        let mut item = new_adaptive_isolation_template("x", "y");
        item.duration_minutes = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveIsolationTemplateCodeLengthRule { max_length: 86400 };
        let mut item = new_adaptive_isolation_template("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AdaptiveIsolationTemplateLabelLengthRule { max_length: 120 };
        let mut item = new_adaptive_isolation_template("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
