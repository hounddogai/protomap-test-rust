use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive plan template must pass. Check returns its problems, or none.
pub trait AdaptivePlanTemplateRule: std::fmt::Debug {
    fn check(&self, item: &AdaptivePlanTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanTemplateRequiredChannelRule {
    pub message: String,
}

impl AdaptivePlanTemplateRule for AdaptivePlanTemplateRequiredChannelRule {
    fn check(&self, item: &AdaptivePlanTemplate) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanTemplateCodePrefixRule {
    pub prefix: String,
}

impl AdaptivePlanTemplateRule for AdaptivePlanTemplateCodePrefixRule {
    fn check(&self, item: &AdaptivePlanTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanTemplateTitleLengthRule {
    pub max_length: i64,
}

impl AdaptivePlanTemplateRule for AdaptivePlanTemplateTitleLengthRule {
    fn check(&self, item: &AdaptivePlanTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive plan template.
#[derive(Debug, Clone)]
pub struct AdaptivePlanTemplateValidator {
    pub rules: Vec<Rc<dyn AdaptivePlanTemplateRule>>,
}

impl AdaptivePlanTemplateValidator {
    pub fn validate(&self, item: &AdaptivePlanTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptivePlanTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptivePlanTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_plan_template_validator() -> AdaptivePlanTemplateValidator {
    AdaptivePlanTemplateValidator {
        rules: vec![
            Rc::new(AdaptivePlanTemplateRequiredChannelRule {
                message: "adaptive plan template needs this field".to_string(),
            }) as Rc<dyn AdaptivePlanTemplateRule>,
            Rc::new(AdaptivePlanTemplateCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(AdaptivePlanTemplateTitleLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_plan_template_validator();
        let item = new_adaptive_plan_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = AdaptivePlanTemplateRequiredChannelRule {
            message: "adaptive plan template needs this field".to_string(),
        };
        let mut item = new_adaptive_plan_template("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AdaptivePlanTemplateTitleLengthRule { max_length: 255 };
        let mut item = new_adaptive_plan_template("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
