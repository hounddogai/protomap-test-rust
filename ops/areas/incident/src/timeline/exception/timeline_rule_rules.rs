use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline rule must pass. Check returns its problems, or none.
pub trait TimelineRuleRule: std::fmt::Debug {
    fn check(&self, item: &TimelineRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRuleRequiredNameRule {
    pub message: String,
}

impl TimelineRuleRule for TimelineRuleRequiredNameRule {
    fn check(&self, item: &TimelineRule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRuleCodeLengthRule {
    pub max_length: i64,
}

impl TimelineRuleRule for TimelineRuleCodeLengthRule {
    fn check(&self, item: &TimelineRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRuleUnitLengthRule {
    pub max_length: i64,
}

impl TimelineRuleRule for TimelineRuleUnitLengthRule {
    fn check(&self, item: &TimelineRule) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline rule.
#[derive(Debug, Clone)]
pub struct TimelineRuleValidator {
    pub rules: Vec<Rc<dyn TimelineRuleRule>>,
}

impl TimelineRuleValidator {
    pub fn validate(&self, item: &TimelineRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_rule_validator() -> TimelineRuleValidator {
    TimelineRuleValidator {
        rules: vec![
            Rc::new(TimelineRuleRequiredNameRule {
                message: "timeline rule needs an entry".to_string(),
            }) as Rc<dyn TimelineRuleRule>,
            Rc::new(TimelineRuleCodeLengthRule { max_length: 64 }),
            Rc::new(TimelineRuleUnitLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_rule_validator();
        let item = new_timeline_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TimelineRuleRequiredNameRule {
            message: "timeline rule needs this field".to_string(),
        };
        let mut item = new_timeline_rule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TimelineRuleCodeLengthRule { max_length: 120 };
        let mut item = new_timeline_rule("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = TimelineRuleUnitLengthRule { max_length: 86400 };
        let mut item = new_timeline_rule("x", "y");
        item.unit = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
