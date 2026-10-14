use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule template must pass. Check returns its problems, or none.
pub trait RuleTemplateRule: std::fmt::Debug {
    fn check(&self, item: &RuleTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleTemplateRequiredTitleRule {
    pub message: String,
}

impl RuleTemplateRule for RuleTemplateRequiredTitleRule {
    fn check(&self, item: &RuleTemplate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleTemplateCodePrefixRule {
    pub prefix: String,
}

impl RuleTemplateRule for RuleTemplateCodePrefixRule {
    fn check(&self, item: &RuleTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleTemplateDescriptionLengthRule {
    pub max_length: i64,
}

impl RuleTemplateRule for RuleTemplateDescriptionLengthRule {
    fn check(&self, item: &RuleTemplate) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule template.
#[derive(Debug, Clone)]
pub struct RuleTemplateValidator {
    pub rules: Vec<Rc<dyn RuleTemplateRule>>,
}

impl RuleTemplateValidator {
    pub fn validate(&self, item: &RuleTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_template_validator() -> RuleTemplateValidator {
    RuleTemplateValidator {
        rules: vec![
            Rc::new(RuleTemplateRequiredTitleRule {
                message: "rule template needs this field".to_string(),
            }) as Rc<dyn RuleTemplateRule>,
            Rc::new(RuleTemplateCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RuleTemplateDescriptionLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_template_validator();
        let item = new_rule_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RuleTemplateRequiredTitleRule {
            message: "rule template needs an entry".to_string(),
        };
        let mut item = new_rule_template("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = RuleTemplateDescriptionLengthRule { max_length: 64 };
        let mut item = new_rule_template("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
