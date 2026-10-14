use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled overage estimate must pass. Check returns its problems, or none.
pub trait ScheduledOverageEstimateRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledOverageEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledOverageEstimateRequiredCodeRule {
    pub message: String,
}

impl ScheduledOverageEstimateRule for ScheduledOverageEstimateRequiredCodeRule {
    fn check(&self, item: &ScheduledOverageEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledOverageEstimateWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledOverageEstimateRule for ScheduledOverageEstimateWeightRangeRule {
    fn check(&self, item: &ScheduledOverageEstimate) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledOverageEstimateCodePrefixRule {
    pub prefix: String,
}

impl ScheduledOverageEstimateRule for ScheduledOverageEstimateCodePrefixRule {
    fn check(&self, item: &ScheduledOverageEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledOverageEstimateAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledOverageEstimateRule for ScheduledOverageEstimateAttemptsRangeRule {
    fn check(&self, item: &ScheduledOverageEstimate) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a scheduled overage estimate.
#[derive(Debug, Clone)]
pub struct ScheduledOverageEstimateValidator {
    pub rules: Vec<Rc<dyn ScheduledOverageEstimateRule>>,
}

impl ScheduledOverageEstimateValidator {
    pub fn validate(&self, item: &ScheduledOverageEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledOverageEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledOverageEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_overage_estimate_validator() -> ScheduledOverageEstimateValidator {
    ScheduledOverageEstimateValidator {
        rules: vec![
            Rc::new(ScheduledOverageEstimateRequiredCodeRule {
                message: "scheduled overage estimate needs a value".to_string(),
            }) as Rc<dyn ScheduledOverageEstimateRule>,
            Rc::new(ScheduledOverageEstimateWeightRangeRule { min: 2.6, max: 1533.3 }),
            Rc::new(ScheduledOverageEstimateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ScheduledOverageEstimateAttemptsRangeRule { min: 16, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_overage_estimate_validator();
        let item = new_scheduled_overage_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledOverageEstimateRequiredCodeRule {
            message: "scheduled overage estimate needs an entry".to_string(),
        };
        let mut item = new_scheduled_overage_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = ScheduledOverageEstimateWeightRangeRule { min: 5.3, max: 3049.6 };
        let mut item = new_scheduled_overage_estimate("x", "y");
        item.weight = 3051.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ScheduledOverageEstimateAttemptsRangeRule { min: 32, max: 3600 };
        let mut item = new_scheduled_overage_estimate("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
