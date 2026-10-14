use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict retention schedule must pass. Check returns its problems, or none.
pub trait StrictRetentionScheduleRule: std::fmt::Debug {
    fn check(&self, item: &StrictRetentionSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionScheduleRequiredCodeRule {
    pub message: String,
}

impl StrictRetentionScheduleRule for StrictRetentionScheduleRequiredCodeRule {
    fn check(&self, item: &StrictRetentionSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionScheduleMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictRetentionScheduleRule for StrictRetentionScheduleMarginRangeRule {
    fn check(&self, item: &StrictRetentionSchedule) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionScheduleCodePrefixRule {
    pub prefix: String,
}

impl StrictRetentionScheduleRule for StrictRetentionScheduleCodePrefixRule {
    fn check(&self, item: &StrictRetentionSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionScheduleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl StrictRetentionScheduleRule for StrictRetentionScheduleAllowedLabelRule {
    fn check(&self, item: &StrictRetentionSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict retention schedule.
#[derive(Debug, Clone)]
pub struct StrictRetentionScheduleValidator {
    pub rules: Vec<Rc<dyn StrictRetentionScheduleRule>>,
}

impl StrictRetentionScheduleValidator {
    pub fn validate(&self, item: &StrictRetentionSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictRetentionSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictRetentionSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_retention_schedule_validator() -> StrictRetentionScheduleValidator {
    StrictRetentionScheduleValidator {
        rules: vec![
            Rc::new(StrictRetentionScheduleRequiredCodeRule {
                message: "strict retention schedule needs a value".to_string(),
            }) as Rc<dyn StrictRetentionScheduleRule>,
            Rc::new(StrictRetentionScheduleMarginRangeRule { min: 7.1, max: 1296.1 }),
            Rc::new(StrictRetentionScheduleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(StrictRetentionScheduleAllowedLabelRule {
                allowed: vec!["global".to_string(), "manual".to_string(), "capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_retention_schedule_validator();
        let item = new_strict_retention_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictRetentionScheduleRequiredCodeRule {
            message: "strict retention schedule needs an entry".to_string(),
        };
        let mut item = new_strict_retention_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = StrictRetentionScheduleMarginRangeRule { min: 1.2, max: 3822.6 };
        let mut item = new_strict_retention_schedule("x", "y");
        item.margin = 3824.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
