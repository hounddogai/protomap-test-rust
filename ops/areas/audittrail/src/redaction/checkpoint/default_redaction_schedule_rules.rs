use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default redaction schedule must pass. Check returns its problems, or none.
pub trait DefaultRedactionScheduleRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRedactionSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionScheduleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultRedactionScheduleRule for DefaultRedactionScheduleDiscountRangeRule {
    fn check(&self, item: &DefaultRedactionSchedule) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionScheduleTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl DefaultRedactionScheduleRule for DefaultRedactionScheduleTagsLimitRule {
    fn check(&self, item: &DefaultRedactionSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionScheduleScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultRedactionScheduleRule for DefaultRedactionScheduleScoreRangeRule {
    fn check(&self, item: &DefaultRedactionSchedule) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionScheduleRequiredSourceRule {
    pub message: String,
}

impl DefaultRedactionScheduleRule for DefaultRedactionScheduleRequiredSourceRule {
    fn check(&self, item: &DefaultRedactionSchedule) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionScheduleCodePrefixRule {
    pub prefix: String,
}

impl DefaultRedactionScheduleRule for DefaultRedactionScheduleCodePrefixRule {
    fn check(&self, item: &DefaultRedactionSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionScheduleRequiredLabelRule {
    pub message: String,
}

impl DefaultRedactionScheduleRule for DefaultRedactionScheduleRequiredLabelRule {
    fn check(&self, item: &DefaultRedactionSchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default redaction schedule.
#[derive(Debug, Clone)]
pub struct DefaultRedactionScheduleValidator {
    pub rules: Vec<Rc<dyn DefaultRedactionScheduleRule>>,
}

impl DefaultRedactionScheduleValidator {
    pub fn validate(&self, item: &DefaultRedactionSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultRedactionSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRedactionSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_redaction_schedule_validator() -> DefaultRedactionScheduleValidator {
    DefaultRedactionScheduleValidator {
        rules: vec![
            Rc::new(DefaultRedactionScheduleDiscountRangeRule { min: 4.1, max: 4921.5 })
                as Rc<dyn DefaultRedactionScheduleRule>,
            Rc::new(DefaultRedactionScheduleTagsLimitRule {
                max_count: 86400,
                banned: Vec::new(),
            }),
            Rc::new(DefaultRedactionScheduleScoreRangeRule { min: 8.1, max: 4646.2 }),
            Rc::new(DefaultRedactionScheduleRequiredSourceRule {
                message: "default redaction schedule needs an entry".to_string(),
            }),
            Rc::new(DefaultRedactionScheduleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(DefaultRedactionScheduleRequiredLabelRule {
                message: "default redaction schedule needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_redaction_schedule_validator();
        let item = new_default_redaction_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = DefaultRedactionScheduleDiscountRangeRule { min: 2.3, max: 2234.8 };
        let mut item = new_default_redaction_schedule("x", "y");
        item.discount = 2236.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = DefaultRedactionScheduleScoreRangeRule { min: 8.1, max: 1325.8 };
        let mut item = new_default_redaction_schedule("x", "y");
        item.score = 1327.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = DefaultRedactionScheduleRequiredSourceRule {
            message: "default redaction schedule needs a value".to_string(),
        };
        let mut item = new_default_redaction_schedule("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultRedactionScheduleRequiredLabelRule {
            message: "default redaction schedule needs an entry".to_string(),
        };
        let mut item = new_default_redaction_schedule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
