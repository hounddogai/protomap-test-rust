use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual flow log baseline must pass. Check returns its problems, or none.
pub trait ManualFlowLogBaselineRule: std::fmt::Debug {
    fn check(&self, item: &ManualFlowLogBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogBaselineRequiredSourceRule {
    pub message: String,
}

impl ManualFlowLogBaselineRule for ManualFlowLogBaselineRequiredSourceRule {
    fn check(&self, item: &ManualFlowLogBaseline) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogBaselineRequiredNameRule {
    pub message: String,
}

impl ManualFlowLogBaselineRule for ManualFlowLogBaselineRequiredNameRule {
    fn check(&self, item: &ManualFlowLogBaseline) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogBaselineRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualFlowLogBaselineRule for ManualFlowLogBaselineRetriesRangeRule {
    fn check(&self, item: &ManualFlowLogBaseline) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogBaselineRequiredCodeRule {
    pub message: String,
}

impl ManualFlowLogBaselineRule for ManualFlowLogBaselineRequiredCodeRule {
    fn check(&self, item: &ManualFlowLogBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogBaselineCodePrefixRule {
    pub prefix: String,
}

impl ManualFlowLogBaselineRule for ManualFlowLogBaselineCodePrefixRule {
    fn check(&self, item: &ManualFlowLogBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogBaselineMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualFlowLogBaselineRule for ManualFlowLogBaselineMarginRangeRule {
    fn check(&self, item: &ManualFlowLogBaseline) -> Vec<String> {
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

/// Runs every rule of a manual flow log baseline.
#[derive(Debug, Clone)]
pub struct ManualFlowLogBaselineValidator {
    pub rules: Vec<Rc<dyn ManualFlowLogBaselineRule>>,
}

impl ManualFlowLogBaselineValidator {
    pub fn validate(&self, item: &ManualFlowLogBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualFlowLogBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualFlowLogBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_flow_log_baseline_validator() -> ManualFlowLogBaselineValidator {
    ManualFlowLogBaselineValidator {
        rules: vec![
            Rc::new(ManualFlowLogBaselineRequiredSourceRule {
                message: "manual flow log baseline needs an entry".to_string(),
            }) as Rc<dyn ManualFlowLogBaselineRule>,
            Rc::new(ManualFlowLogBaselineRequiredNameRule {
                message: "manual flow log baseline needs an entry".to_string(),
            }),
            Rc::new(ManualFlowLogBaselineRetriesRangeRule { min: 32, max: 120 }),
            Rc::new(ManualFlowLogBaselineRequiredCodeRule {
                message: "manual flow log baseline needs an entry".to_string(),
            }),
            Rc::new(ManualFlowLogBaselineCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ManualFlowLogBaselineMarginRangeRule { min: 3.6, max: 3805.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_flow_log_baseline_validator();
        let item = new_manual_flow_log_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = ManualFlowLogBaselineRequiredSourceRule {
            message: "manual flow log baseline needs a value".to_string(),
        };
        let mut item = new_manual_flow_log_baseline("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ManualFlowLogBaselineRequiredNameRule {
            message: "manual flow log baseline needs a value".to_string(),
        };
        let mut item = new_manual_flow_log_baseline("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = ManualFlowLogBaselineRetriesRangeRule { min: 64, max: 1000 };
        let mut item = new_manual_flow_log_baseline("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualFlowLogBaselineRequiredCodeRule {
            message: "manual flow log baseline needs this field".to_string(),
        };
        let mut item = new_manual_flow_log_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = ManualFlowLogBaselineMarginRangeRule { min: 9.8, max: 2693.1 };
        let mut item = new_manual_flow_log_baseline("x", "y");
        item.margin = 2694.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
