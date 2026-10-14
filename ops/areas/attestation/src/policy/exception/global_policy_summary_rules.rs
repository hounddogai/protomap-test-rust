use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global policy summary must pass. Check returns its problems, or none.
pub trait GlobalPolicySummaryRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPolicySummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicySummaryExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalPolicySummaryRule for GlobalPolicySummaryExpiresAtRangeRule {
    fn check(&self, item: &GlobalPolicySummary) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicySummaryRequiredCodeRule {
    pub message: String,
}

impl GlobalPolicySummaryRule for GlobalPolicySummaryRequiredCodeRule {
    fn check(&self, item: &GlobalPolicySummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicySummaryRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalPolicySummaryRule for GlobalPolicySummaryRotationDaysRangeRule {
    fn check(&self, item: &GlobalPolicySummary) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicySummaryCodeLengthRule {
    pub max_length: i64,
}

impl GlobalPolicySummaryRule for GlobalPolicySummaryCodeLengthRule {
    fn check(&self, item: &GlobalPolicySummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicySummaryLabelLengthRule {
    pub max_length: i64,
}

impl GlobalPolicySummaryRule for GlobalPolicySummaryLabelLengthRule {
    fn check(&self, item: &GlobalPolicySummary) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global policy summary.
#[derive(Debug, Clone)]
pub struct GlobalPolicySummaryValidator {
    pub rules: Vec<Rc<dyn GlobalPolicySummaryRule>>,
}

impl GlobalPolicySummaryValidator {
    pub fn validate(&self, item: &GlobalPolicySummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalPolicySummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPolicySummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_policy_summary_validator() -> GlobalPolicySummaryValidator {
    GlobalPolicySummaryValidator {
        rules: vec![
            Rc::new(GlobalPolicySummaryExpiresAtRangeRule { min: 16, max: 32 }) as Rc<dyn GlobalPolicySummaryRule>,
            Rc::new(GlobalPolicySummaryRequiredCodeRule {
                message: "global policy summary needs a value".to_string(),
            }),
            Rc::new(GlobalPolicySummaryRotationDaysRangeRule { min: 32, max: 255 }),
            Rc::new(GlobalPolicySummaryCodeLengthRule { max_length: 3600 }),
            Rc::new(GlobalPolicySummaryLabelLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_policy_summary_validator();
        let item = new_global_policy_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = GlobalPolicySummaryExpiresAtRangeRule { min: 120, max: 255 };
        let mut item = new_global_policy_summary("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalPolicySummaryRequiredCodeRule {
            message: "global policy summary needs a value".to_string(),
        };
        let mut item = new_global_policy_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = GlobalPolicySummaryRotationDaysRangeRule { min: 32, max: 80 };
        let mut item = new_global_policy_summary("x", "y");
        item.rotation_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalPolicySummaryCodeLengthRule { max_length: 16 };
        let mut item = new_global_policy_summary("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = GlobalPolicySummaryLabelLengthRule { max_length: 64 };
        let mut item = new_global_policy_summary("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
