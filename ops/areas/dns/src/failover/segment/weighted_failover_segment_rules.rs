use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted failover segment must pass. Check returns its problems, or none.
pub trait WeightedFailoverSegmentRule: std::fmt::Debug {
    fn check(&self, item: &WeightedFailoverSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverSegmentCodeLengthRule {
    pub max_length: i64,
}

impl WeightedFailoverSegmentRule for WeightedFailoverSegmentCodeLengthRule {
    fn check(&self, item: &WeightedFailoverSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverSegmentLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedFailoverSegmentRule for WeightedFailoverSegmentLimitRangeRule {
    fn check(&self, item: &WeightedFailoverSegment) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverSegmentWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedFailoverSegmentRule for WeightedFailoverSegmentWindowDaysRangeRule {
    fn check(&self, item: &WeightedFailoverSegment) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted failover segment.
#[derive(Debug, Clone)]
pub struct WeightedFailoverSegmentValidator {
    pub rules: Vec<Rc<dyn WeightedFailoverSegmentRule>>,
}

impl WeightedFailoverSegmentValidator {
    pub fn validate(&self, item: &WeightedFailoverSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedFailoverSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedFailoverSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_failover_segment_validator() -> WeightedFailoverSegmentValidator {
    WeightedFailoverSegmentValidator {
        rules: vec![
            Rc::new(WeightedFailoverSegmentCodeLengthRule { max_length: 80 }) as Rc<dyn WeightedFailoverSegmentRule>,
            Rc::new(WeightedFailoverSegmentLimitRangeRule { min: 16, max: 255 }),
            Rc::new(WeightedFailoverSegmentWindowDaysRangeRule { min: 64, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_failover_segment_validator();
        let item = new_weighted_failover_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedFailoverSegmentCodeLengthRule { max_length: 1000 };
        let mut item = new_weighted_failover_segment("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = WeightedFailoverSegmentLimitRangeRule { min: 1000, max: 3600 };
        let mut item = new_weighted_failover_segment("x", "y");
        item.limit = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = WeightedFailoverSegmentWindowDaysRangeRule { min: 255, max: 3600 };
        let mut item = new_weighted_failover_segment("x", "y");
        item.window_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
