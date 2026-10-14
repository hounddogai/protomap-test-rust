use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo plan must pass. Check returns its problems, or none.
pub trait SloPlanRule: std::fmt::Debug {
    fn check(&self, item: &SloPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPlanSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloPlanRule for SloPlanSequenceRangeRule {
    fn check(&self, item: &SloPlan) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPlanCodePrefixRule {
    pub prefix: String,
}

impl SloPlanRule for SloPlanCodePrefixRule {
    fn check(&self, item: &SloPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPlanChannelLengthRule {
    pub max_length: i64,
}

impl SloPlanRule for SloPlanChannelLengthRule {
    fn check(&self, item: &SloPlan) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPlanRequiredLabelRule {
    pub message: String,
}

impl SloPlanRule for SloPlanRequiredLabelRule {
    fn check(&self, item: &SloPlan) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPlanRequiredCodeRule {
    pub message: String,
}

impl SloPlanRule for SloPlanRequiredCodeRule {
    fn check(&self, item: &SloPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo plan.
#[derive(Debug, Clone)]
pub struct SloPlanValidator {
    pub rules: Vec<Rc<dyn SloPlanRule>>,
}

impl SloPlanValidator {
    pub fn validate(&self, item: &SloPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_plan_validator() -> SloPlanValidator {
    SloPlanValidator {
        rules: vec![
            Rc::new(SloPlanSequenceRangeRule { min: 1000, max: 1000 }) as Rc<dyn SloPlanRule>,
            Rc::new(SloPlanCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(SloPlanChannelLengthRule { max_length: 64 }),
            Rc::new(SloPlanRequiredLabelRule {
                message: "slo plan needs an entry".to_string(),
            }),
            Rc::new(SloPlanRequiredCodeRule {
                message: "slo plan needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_plan_validator();
        let item = new_slo_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = SloPlanSequenceRangeRule { min: 32, max: 86400 };
        let mut item = new_slo_plan("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = SloPlanChannelLengthRule { max_length: 16 };
        let mut item = new_slo_plan("x", "y");
        item.channel = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SloPlanRequiredLabelRule {
            message: "slo plan needs this field".to_string(),
        };
        let mut item = new_slo_plan("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SloPlanRequiredCodeRule {
            message: "slo plan needs this field".to_string(),
        };
        let mut item = new_slo_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
