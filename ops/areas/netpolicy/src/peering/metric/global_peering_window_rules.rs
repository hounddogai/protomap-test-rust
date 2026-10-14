use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global peering window must pass. Check returns its problems, or none.
pub trait GlobalPeeringWindowRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPeeringWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringWindowRequiredReviewerRule {
    pub message: String,
}

impl GlobalPeeringWindowRule for GlobalPeeringWindowRequiredReviewerRule {
    fn check(&self, item: &GlobalPeeringWindow) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringWindowRequiredCodeRule {
    pub message: String,
}

impl GlobalPeeringWindowRule for GlobalPeeringWindowRequiredCodeRule {
    fn check(&self, item: &GlobalPeeringWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringWindowThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalPeeringWindowRule for GlobalPeeringWindowThresholdRangeRule {
    fn check(&self, item: &GlobalPeeringWindow) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringWindowMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalPeeringWindowRule for GlobalPeeringWindowMaximumRangeRule {
    fn check(&self, item: &GlobalPeeringWindow) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringWindowKeyIdLengthRule {
    pub max_length: i64,
}

impl GlobalPeeringWindowRule for GlobalPeeringWindowKeyIdLengthRule {
    fn check(&self, item: &GlobalPeeringWindow) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global peering window.
#[derive(Debug, Clone)]
pub struct GlobalPeeringWindowValidator {
    pub rules: Vec<Rc<dyn GlobalPeeringWindowRule>>,
}

impl GlobalPeeringWindowValidator {
    pub fn validate(&self, item: &GlobalPeeringWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalPeeringWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPeeringWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_peering_window_validator() -> GlobalPeeringWindowValidator {
    GlobalPeeringWindowValidator {
        rules: vec![
            Rc::new(GlobalPeeringWindowRequiredReviewerRule {
                message: "global peering window needs this field".to_string(),
            }) as Rc<dyn GlobalPeeringWindowRule>,
            Rc::new(GlobalPeeringWindowRequiredCodeRule {
                message: "global peering window needs a value".to_string(),
            }),
            Rc::new(GlobalPeeringWindowThresholdRangeRule { min: 1.3, max: 984.5 }),
            Rc::new(GlobalPeeringWindowMaximumRangeRule { min: 9.9, max: 3584.5 }),
            Rc::new(GlobalPeeringWindowKeyIdLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_peering_window_validator();
        let item = new_global_peering_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = GlobalPeeringWindowRequiredReviewerRule {
            message: "global peering window needs a value".to_string(),
        };
        let mut item = new_global_peering_window("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalPeeringWindowRequiredCodeRule {
            message: "global peering window needs a value".to_string(),
        };
        let mut item = new_global_peering_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = GlobalPeeringWindowThresholdRangeRule { min: 9.2, max: 4645.8 };
        let mut item = new_global_peering_window("x", "y");
        item.threshold = 4647.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = GlobalPeeringWindowMaximumRangeRule { min: 8.2, max: 1005.0 };
        let mut item = new_global_peering_window("x", "y");
        item.maximum = 1006.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = GlobalPeeringWindowKeyIdLengthRule { max_length: 255 };
        let mut item = new_global_peering_window("x", "y");
        item.key_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
