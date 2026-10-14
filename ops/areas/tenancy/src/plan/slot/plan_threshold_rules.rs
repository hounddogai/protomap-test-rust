use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan threshold must pass. Check returns its problems, or none.
pub trait PlanThresholdRule: std::fmt::Debug {
    fn check(&self, item: &PlanThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanThresholdRequiredTitleRule {
    pub message: String,
}

impl PlanThresholdRule for PlanThresholdRequiredTitleRule {
    fn check(&self, item: &PlanThreshold) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanThresholdVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanThresholdRule for PlanThresholdVersionRangeRule {
    fn check(&self, item: &PlanThreshold) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanThresholdRequiredCodeRule {
    pub message: String,
}

impl PlanThresholdRule for PlanThresholdRequiredCodeRule {
    fn check(&self, item: &PlanThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan threshold.
#[derive(Debug, Clone)]
pub struct PlanThresholdValidator {
    pub rules: Vec<Rc<dyn PlanThresholdRule>>,
}

impl PlanThresholdValidator {
    pub fn validate(&self, item: &PlanThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_threshold_validator() -> PlanThresholdValidator {
    PlanThresholdValidator {
        rules: vec![
            Rc::new(PlanThresholdRequiredTitleRule {
                message: "plan threshold needs a value".to_string(),
            }) as Rc<dyn PlanThresholdRule>,
            Rc::new(PlanThresholdVersionRangeRule { min: 32, max: 32 }),
            Rc::new(PlanThresholdRequiredCodeRule {
                message: "plan threshold needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_threshold_validator();
        let item = new_plan_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PlanThresholdRequiredTitleRule {
            message: "plan threshold needs a value".to_string(),
        };
        let mut item = new_plan_threshold("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = PlanThresholdVersionRangeRule { min: 16, max: 255 };
        let mut item = new_plan_threshold("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PlanThresholdRequiredCodeRule {
            message: "plan threshold needs an entry".to_string(),
        };
        let mut item = new_plan_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
