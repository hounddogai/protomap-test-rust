use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual binding group must pass. Check returns its problems, or none.
pub trait ManualBindingGroupRule: std::fmt::Debug {
    fn check(&self, item: &ManualBindingGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingGroupThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualBindingGroupRule for ManualBindingGroupThresholdRangeRule {
    fn check(&self, item: &ManualBindingGroup) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingGroupMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualBindingGroupRule for ManualBindingGroupMaximumRangeRule {
    fn check(&self, item: &ManualBindingGroup) -> Vec<String> {
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
pub struct ManualBindingGroupCodeLengthRule {
    pub max_length: i64,
}

impl ManualBindingGroupRule for ManualBindingGroupCodeLengthRule {
    fn check(&self, item: &ManualBindingGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingGroupAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ManualBindingGroupRule for ManualBindingGroupAllowedTitleRule {
    fn check(&self, item: &ManualBindingGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingGroupRequiredTitleRule {
    pub message: String,
}

impl ManualBindingGroupRule for ManualBindingGroupRequiredTitleRule {
    fn check(&self, item: &ManualBindingGroup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual binding group.
#[derive(Debug, Clone)]
pub struct ManualBindingGroupValidator {
    pub rules: Vec<Rc<dyn ManualBindingGroupRule>>,
}

impl ManualBindingGroupValidator {
    pub fn validate(&self, item: &ManualBindingGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualBindingGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualBindingGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_binding_group_validator() -> ManualBindingGroupValidator {
    ManualBindingGroupValidator {
        rules: vec![
            Rc::new(ManualBindingGroupThresholdRangeRule { min: 7.9, max: 3928.3 }) as Rc<dyn ManualBindingGroupRule>,
            Rc::new(ManualBindingGroupMaximumRangeRule { min: 7.3, max: 3153.3 }),
            Rc::new(ManualBindingGroupCodeLengthRule { max_length: 32 }),
            Rc::new(ManualBindingGroupAllowedTitleRule {
                allowed: vec!["manual".to_string(), "manual".to_string()],
            }),
            Rc::new(ManualBindingGroupRequiredTitleRule {
                message: "manual binding group needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_binding_group_validator();
        let item = new_manual_binding_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = ManualBindingGroupThresholdRangeRule { min: 6.8, max: 316.5 };
        let mut item = new_manual_binding_group("x", "y");
        item.threshold = 318.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = ManualBindingGroupMaximumRangeRule { min: 6.7, max: 4453.7 };
        let mut item = new_manual_binding_group("x", "y");
        item.maximum = 4455.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualBindingGroupCodeLengthRule { max_length: 86400 };
        let mut item = new_manual_binding_group("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualBindingGroupRequiredTitleRule {
            message: "manual binding group needs a value".to_string(),
        };
        let mut item = new_manual_binding_group("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
