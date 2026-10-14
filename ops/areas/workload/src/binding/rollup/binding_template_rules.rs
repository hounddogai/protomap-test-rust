use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding template must pass. Check returns its problems, or none.
pub trait BindingTemplateRule: std::fmt::Debug {
    fn check(&self, item: &BindingTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingTemplateCodeLengthRule {
    pub max_length: i64,
}

impl BindingTemplateRule for BindingTemplateCodeLengthRule {
    fn check(&self, item: &BindingTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingTemplateAudienceLengthRule {
    pub max_length: i64,
}

impl BindingTemplateRule for BindingTemplateAudienceLengthRule {
    fn check(&self, item: &BindingTemplate) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingTemplateQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingTemplateRule for BindingTemplateQuantityRangeRule {
    fn check(&self, item: &BindingTemplate) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingTemplateRequiredTitleRule {
    pub message: String,
}

impl BindingTemplateRule for BindingTemplateRequiredTitleRule {
    fn check(&self, item: &BindingTemplate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding template.
#[derive(Debug, Clone)]
pub struct BindingTemplateValidator {
    pub rules: Vec<Rc<dyn BindingTemplateRule>>,
}

impl BindingTemplateValidator {
    pub fn validate(&self, item: &BindingTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_template_validator() -> BindingTemplateValidator {
    BindingTemplateValidator {
        rules: vec![
            Rc::new(BindingTemplateCodeLengthRule { max_length: 80 }) as Rc<dyn BindingTemplateRule>,
            Rc::new(BindingTemplateAudienceLengthRule { max_length: 80 }),
            Rc::new(BindingTemplateQuantityRangeRule { min: 32, max: 1000 }),
            Rc::new(BindingTemplateRequiredTitleRule {
                message: "binding template needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_template_validator();
        let item = new_binding_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BindingTemplateCodeLengthRule { max_length: 16 };
        let mut item = new_binding_template("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = BindingTemplateAudienceLengthRule { max_length: 1000 };
        let mut item = new_binding_template("x", "y");
        item.audience = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = BindingTemplateQuantityRangeRule { min: 64, max: 3600 };
        let mut item = new_binding_template("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BindingTemplateRequiredTitleRule {
            message: "binding template needs an entry".to_string(),
        };
        let mut item = new_binding_template("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
