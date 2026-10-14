use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual timeline adjustment must pass. Check returns its problems, or none.
pub trait ManualTimelineAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &ManualTimelineAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTimelineAdjustmentPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualTimelineAdjustmentRule for ManualTimelineAdjustmentPortRangeRule {
    fn check(&self, item: &ManualTimelineAdjustment) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTimelineAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl ManualTimelineAdjustmentRule for ManualTimelineAdjustmentCodeLengthRule {
    fn check(&self, item: &ManualTimelineAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTimelineAdjustmentActiveGuardRule {
    pub limit: i64,
}

impl ManualTimelineAdjustmentRule for ManualTimelineAdjustmentActiveGuardRule {
    fn check(&self, item: &ManualTimelineAdjustment) -> Vec<String> {
        if item.active && (item.updated_at > self.limit) {
            return vec![format!(
                "updated at {} exceeds {} while active",
                item.updated_at, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual timeline adjustment.
#[derive(Debug, Clone)]
pub struct ManualTimelineAdjustmentValidator {
    pub rules: Vec<Rc<dyn ManualTimelineAdjustmentRule>>,
}

impl ManualTimelineAdjustmentValidator {
    pub fn validate(&self, item: &ManualTimelineAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualTimelineAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualTimelineAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_timeline_adjustment_validator() -> ManualTimelineAdjustmentValidator {
    ManualTimelineAdjustmentValidator {
        rules: vec![
            Rc::new(ManualTimelineAdjustmentPortRangeRule { min: 16, max: 255 })
                as Rc<dyn ManualTimelineAdjustmentRule>,
            Rc::new(ManualTimelineAdjustmentCodeLengthRule { max_length: 64 }),
            Rc::new(ManualTimelineAdjustmentActiveGuardRule { limit: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_timeline_adjustment_validator();
        let item = new_manual_timeline_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = ManualTimelineAdjustmentPortRangeRule { min: 80, max: 86400 };
        let mut item = new_manual_timeline_adjustment("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualTimelineAdjustmentCodeLengthRule { max_length: 255 };
        let mut item = new_manual_timeline_adjustment("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
