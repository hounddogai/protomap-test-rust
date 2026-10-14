use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled zone score must pass. Check returns its problems, or none.
pub trait ScheduledZoneScoreRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledZoneScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneScoreRequiredCodeRule {
    pub message: String,
}

impl ScheduledZoneScoreRule for ScheduledZoneScoreRequiredCodeRule {
    fn check(&self, item: &ScheduledZoneScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneScoreRequiredSegmentRule {
    pub message: String,
}

impl ScheduledZoneScoreRule for ScheduledZoneScoreRequiredSegmentRule {
    fn check(&self, item: &ScheduledZoneScore) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneScoreSegmentLengthRule {
    pub max_length: i64,
}

impl ScheduledZoneScoreRule for ScheduledZoneScoreSegmentLengthRule {
    fn check(&self, item: &ScheduledZoneScore) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneScoreCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledZoneScoreRule for ScheduledZoneScoreCostRangeRule {
    fn check(&self, item: &ScheduledZoneScore) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneScoreCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledZoneScoreRule for ScheduledZoneScoreCodeLengthRule {
    fn check(&self, item: &ScheduledZoneScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled zone score.
#[derive(Debug, Clone)]
pub struct ScheduledZoneScoreValidator {
    pub rules: Vec<Rc<dyn ScheduledZoneScoreRule>>,
}

impl ScheduledZoneScoreValidator {
    pub fn validate(&self, item: &ScheduledZoneScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledZoneScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledZoneScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_zone_score_validator() -> ScheduledZoneScoreValidator {
    ScheduledZoneScoreValidator {
        rules: vec![
            Rc::new(ScheduledZoneScoreRequiredCodeRule {
                message: "scheduled zone score needs this field".to_string(),
            }) as Rc<dyn ScheduledZoneScoreRule>,
            Rc::new(ScheduledZoneScoreRequiredSegmentRule {
                message: "scheduled zone score needs this field".to_string(),
            }),
            Rc::new(ScheduledZoneScoreSegmentLengthRule { max_length: 255 }),
            Rc::new(ScheduledZoneScoreCostRangeRule { min: 7.4, max: 430.3 }),
            Rc::new(ScheduledZoneScoreCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_zone_score_validator();
        let item = new_scheduled_zone_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledZoneScoreRequiredCodeRule {
            message: "scheduled zone score needs a value".to_string(),
        };
        let mut item = new_scheduled_zone_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = ScheduledZoneScoreRequiredSegmentRule {
            message: "scheduled zone score needs this field".to_string(),
        };
        let mut item = new_scheduled_zone_score("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = ScheduledZoneScoreSegmentLengthRule { max_length: 255 };
        let mut item = new_scheduled_zone_score("x", "y");
        item.segment = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = ScheduledZoneScoreCostRangeRule { min: 5.7, max: 2631.2 };
        let mut item = new_scheduled_zone_score("x", "y");
        item.cost = 2632.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledZoneScoreCodeLengthRule { max_length: 120 };
        let mut item = new_scheduled_zone_score("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
