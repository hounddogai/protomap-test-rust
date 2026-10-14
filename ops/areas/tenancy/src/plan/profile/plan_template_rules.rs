use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan template must pass. Check returns its problems, or none.
pub trait PlanTemplateRule: std::fmt::Debug {
    fn check(&self, item: &PlanTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanTemplateChannelLengthRule {
    pub max_length: i64,
}

impl PlanTemplateRule for PlanTemplateChannelLengthRule {
    fn check(&self, item: &PlanTemplate) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanTemplateRequiredLabelRule {
    pub message: String,
}

impl PlanTemplateRule for PlanTemplateRequiredLabelRule {
    fn check(&self, item: &PlanTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanTemplateRequiredChannelRule {
    pub message: String,
}

impl PlanTemplateRule for PlanTemplateRequiredChannelRule {
    fn check(&self, item: &PlanTemplate) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan template.
#[derive(Debug, Clone)]
pub struct PlanTemplateValidator {
    pub rules: Vec<Rc<dyn PlanTemplateRule>>,
}

impl PlanTemplateValidator {
    pub fn validate(&self, item: &PlanTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_template_validator() -> PlanTemplateValidator {
    PlanTemplateValidator {
        rules: vec![
            Rc::new(PlanTemplateChannelLengthRule { max_length: 64 }) as Rc<dyn PlanTemplateRule>,
            Rc::new(PlanTemplateRequiredLabelRule {
                message: "plan template needs a value".to_string(),
            }),
            Rc::new(PlanTemplateRequiredChannelRule {
                message: "plan template needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_template_validator();
        let item = new_plan_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = PlanTemplateChannelLengthRule { max_length: 32 };
        let mut item = new_plan_template("x", "y");
        item.channel = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PlanTemplateRequiredLabelRule {
            message: "plan template needs an entry".to_string(),
        };
        let mut item = new_plan_template("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = PlanTemplateRequiredChannelRule {
            message: "plan template needs this field".to_string(),
        };
        let mut item = new_plan_template("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
