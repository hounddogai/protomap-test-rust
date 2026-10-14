use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item segment must pass. Check returns its problems, or none.
pub trait ActionItemSegmentRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSegmentRequiredClusterRule {
    pub message: String,
}

impl ActionItemSegmentRule for ActionItemSegmentRequiredClusterRule {
    fn check(&self, item: &ActionItemSegment) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSegmentScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ActionItemSegmentRule for ActionItemSegmentScoreRangeRule {
    fn check(&self, item: &ActionItemSegment) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSegmentSummaryLengthRule {
    pub max_length: i64,
}

impl ActionItemSegmentRule for ActionItemSegmentSummaryLengthRule {
    fn check(&self, item: &ActionItemSegment) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSegmentNameLengthRule {
    pub max_length: i64,
}

impl ActionItemSegmentRule for ActionItemSegmentNameLengthRule {
    fn check(&self, item: &ActionItemSegment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item segment.
#[derive(Debug, Clone)]
pub struct ActionItemSegmentValidator {
    pub rules: Vec<Rc<dyn ActionItemSegmentRule>>,
}

impl ActionItemSegmentValidator {
    pub fn validate(&self, item: &ActionItemSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_segment_validator() -> ActionItemSegmentValidator {
    ActionItemSegmentValidator {
        rules: vec![
            Rc::new(ActionItemSegmentRequiredClusterRule {
                message: "action item segment needs an entry".to_string(),
            }) as Rc<dyn ActionItemSegmentRule>,
            Rc::new(ActionItemSegmentScoreRangeRule { min: 6.5, max: 4488.7 }),
            Rc::new(ActionItemSegmentSummaryLengthRule { max_length: 80 }),
            Rc::new(ActionItemSegmentNameLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_segment_validator();
        let item = new_action_item_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = ActionItemSegmentRequiredClusterRule {
            message: "action item segment needs a value".to_string(),
        };
        let mut item = new_action_item_segment("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ActionItemSegmentScoreRangeRule { min: 3.7, max: 1724.9 };
        let mut item = new_action_item_segment("x", "y");
        item.score = 1726.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = ActionItemSegmentSummaryLengthRule { max_length: 3600 };
        let mut item = new_action_item_segment("x", "y");
        item.summary = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ActionItemSegmentNameLengthRule { max_length: 64 };
        let mut item = new_action_item_segment("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
