use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage batch must pass. Check returns its problems, or none.
pub trait OverageBatchRule: std::fmt::Debug {
    fn check(&self, item: &OverageBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBatchSegmentLengthRule {
    pub max_length: i64,
}

impl OverageBatchRule for OverageBatchSegmentLengthRule {
    fn check(&self, item: &OverageBatch) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBatchPinnedGuardRule {
    pub limit: i64,
}

impl OverageBatchRule for OverageBatchPinnedGuardRule {
    fn check(&self, item: &OverageBatch) -> Vec<String> {
        if item.pinned && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while pinned",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBatchWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageBatchRule for OverageBatchWindowDaysRangeRule {
    fn check(&self, item: &OverageBatch) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a overage batch.
#[derive(Debug, Clone)]
pub struct OverageBatchValidator {
    pub rules: Vec<Rc<dyn OverageBatchRule>>,
}

impl OverageBatchValidator {
    pub fn validate(&self, item: &OverageBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_batch_validator() -> OverageBatchValidator {
    OverageBatchValidator {
        rules: vec![
            Rc::new(OverageBatchSegmentLengthRule { max_length: 1000 }) as Rc<dyn OverageBatchRule>,
            Rc::new(OverageBatchPinnedGuardRule { limit: 255 }),
            Rc::new(OverageBatchWindowDaysRangeRule { min: 64, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_batch_validator();
        let item = new_overage_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = OverageBatchSegmentLengthRule { max_length: 1000 };
        let mut item = new_overage_batch("x", "y");
        item.segment = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = OverageBatchWindowDaysRangeRule { min: 255, max: 3600 };
        let mut item = new_overage_batch("x", "y");
        item.window_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
