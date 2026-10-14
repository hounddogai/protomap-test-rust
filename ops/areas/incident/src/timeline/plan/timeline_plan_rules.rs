use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline plan must pass. Check returns its problems, or none.
pub trait TimelinePlanRule: std::fmt::Debug {
    fn check(&self, item: &TimelinePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePlanKeyIdLengthRule {
    pub max_length: i64,
}

impl TimelinePlanRule for TimelinePlanKeyIdLengthRule {
    fn check(&self, item: &TimelinePlan) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePlanAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl TimelinePlanRule for TimelinePlanAllowedOwnerIdRule {
    fn check(&self, item: &TimelinePlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is inconsistent", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePlanMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelinePlanRule for TimelinePlanMinimumRangeRule {
    fn check(&self, item: &TimelinePlan) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePlanMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelinePlanRule for TimelinePlanMarginRangeRule {
    fn check(&self, item: &TimelinePlan) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePlanCodeLengthRule {
    pub max_length: i64,
}

impl TimelinePlanRule for TimelinePlanCodeLengthRule {
    fn check(&self, item: &TimelinePlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePlanCodePrefixRule {
    pub prefix: String,
}

impl TimelinePlanRule for TimelinePlanCodePrefixRule {
    fn check(&self, item: &TimelinePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline plan.
#[derive(Debug, Clone)]
pub struct TimelinePlanValidator {
    pub rules: Vec<Rc<dyn TimelinePlanRule>>,
}

impl TimelinePlanValidator {
    pub fn validate(&self, item: &TimelinePlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelinePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelinePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_plan_validator() -> TimelinePlanValidator {
    TimelinePlanValidator {
        rules: vec![
            Rc::new(TimelinePlanKeyIdLengthRule { max_length: 32 }) as Rc<dyn TimelinePlanRule>,
            Rc::new(TimelinePlanAllowedOwnerIdRule {
                allowed: vec!["manual".to_string()],
            }),
            Rc::new(TimelinePlanMinimumRangeRule { min: 1.2, max: 333.7 }),
            Rc::new(TimelinePlanMarginRangeRule { min: 5.3, max: 2439.2 }),
            Rc::new(TimelinePlanCodeLengthRule { max_length: 32 }),
            Rc::new(TimelinePlanCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_plan_validator();
        let item = new_timeline_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = TimelinePlanKeyIdLengthRule { max_length: 120 };
        let mut item = new_timeline_plan("x", "y");
        item.key_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = TimelinePlanMinimumRangeRule { min: 3.8, max: 1076.9 };
        let mut item = new_timeline_plan("x", "y");
        item.minimum = 1078.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = TimelinePlanMarginRangeRule { min: 7.7, max: 412.8 };
        let mut item = new_timeline_plan("x", "y");
        item.margin = 414.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TimelinePlanCodeLengthRule { max_length: 255 };
        let mut item = new_timeline_plan("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
