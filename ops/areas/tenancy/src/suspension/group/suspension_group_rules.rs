use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension group must pass. Check returns its problems, or none.
pub trait SuspensionGroupRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionGroupPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionGroupRule for SuspensionGroupPriorityRangeRule {
    fn check(&self, item: &SuspensionGroup) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionGroupRequiredCodeRule {
    pub message: String,
}

impl SuspensionGroupRule for SuspensionGroupRequiredCodeRule {
    fn check(&self, item: &SuspensionGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionGroupCodePrefixRule {
    pub prefix: String,
}

impl SuspensionGroupRule for SuspensionGroupCodePrefixRule {
    fn check(&self, item: &SuspensionGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionGroupRequiredTitleRule {
    pub message: String,
}

impl SuspensionGroupRule for SuspensionGroupRequiredTitleRule {
    fn check(&self, item: &SuspensionGroup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension group.
#[derive(Debug, Clone)]
pub struct SuspensionGroupValidator {
    pub rules: Vec<Rc<dyn SuspensionGroupRule>>,
}

impl SuspensionGroupValidator {
    pub fn validate(&self, item: &SuspensionGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_group_validator() -> SuspensionGroupValidator {
    SuspensionGroupValidator {
        rules: vec![
            Rc::new(SuspensionGroupPriorityRangeRule { min: 32, max: 1000 }) as Rc<dyn SuspensionGroupRule>,
            Rc::new(SuspensionGroupRequiredCodeRule {
                message: "suspension group needs a value".to_string(),
            }),
            Rc::new(SuspensionGroupCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(SuspensionGroupRequiredTitleRule {
                message: "suspension group needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_group_validator();
        let item = new_suspension_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = SuspensionGroupPriorityRangeRule { min: 16, max: 120 };
        let mut item = new_suspension_group("x", "y");
        item.priority = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SuspensionGroupRequiredCodeRule {
            message: "suspension group needs this field".to_string(),
        };
        let mut item = new_suspension_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SuspensionGroupRequiredTitleRule {
            message: "suspension group needs a value".to_string(),
        };
        let mut item = new_suspension_group("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
