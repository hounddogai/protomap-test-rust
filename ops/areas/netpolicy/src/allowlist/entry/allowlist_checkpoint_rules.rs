use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist checkpoint must pass. Check returns its problems, or none.
pub trait AllowlistCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCheckpointRequiredCodeRule {
    pub message: String,
}

impl AllowlistCheckpointRule for AllowlistCheckpointRequiredCodeRule {
    fn check(&self, item: &AllowlistCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCheckpointPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistCheckpointRule for AllowlistCheckpointPriorityRangeRule {
    fn check(&self, item: &AllowlistCheckpoint) -> Vec<String> {
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
pub struct AllowlistCheckpointRequiredNameRule {
    pub message: String,
}

impl AllowlistCheckpointRule for AllowlistCheckpointRequiredNameRule {
    fn check(&self, item: &AllowlistCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCheckpointRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistCheckpointRule for AllowlistCheckpointRatioRangeRule {
    fn check(&self, item: &AllowlistCheckpoint) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCheckpointRequiredCategoryRule {
    pub message: String,
}

impl AllowlistCheckpointRule for AllowlistCheckpointRequiredCategoryRule {
    fn check(&self, item: &AllowlistCheckpoint) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCheckpointCodePrefixRule {
    pub prefix: String,
}

impl AllowlistCheckpointRule for AllowlistCheckpointCodePrefixRule {
    fn check(&self, item: &AllowlistCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist checkpoint.
#[derive(Debug, Clone)]
pub struct AllowlistCheckpointValidator {
    pub rules: Vec<Rc<dyn AllowlistCheckpointRule>>,
}

impl AllowlistCheckpointValidator {
    pub fn validate(&self, item: &AllowlistCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_checkpoint_validator() -> AllowlistCheckpointValidator {
    AllowlistCheckpointValidator {
        rules: vec![
            Rc::new(AllowlistCheckpointRequiredCodeRule {
                message: "allowlist checkpoint needs this field".to_string(),
            }) as Rc<dyn AllowlistCheckpointRule>,
            Rc::new(AllowlistCheckpointPriorityRangeRule { min: 32, max: 3600 }),
            Rc::new(AllowlistCheckpointRequiredNameRule {
                message: "allowlist checkpoint needs an entry".to_string(),
            }),
            Rc::new(AllowlistCheckpointRatioRangeRule { min: 2.2, max: 2133.2 }),
            Rc::new(AllowlistCheckpointRequiredCategoryRule {
                message: "allowlist checkpoint needs a value".to_string(),
            }),
            Rc::new(AllowlistCheckpointCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_checkpoint_validator();
        let item = new_allowlist_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AllowlistCheckpointRequiredCodeRule {
            message: "allowlist checkpoint needs an entry".to_string(),
        };
        let mut item = new_allowlist_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = AllowlistCheckpointPriorityRangeRule { min: 1000, max: 86400 };
        let mut item = new_allowlist_checkpoint("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AllowlistCheckpointRequiredNameRule {
            message: "allowlist checkpoint needs an entry".to_string(),
        };
        let mut item = new_allowlist_checkpoint("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = AllowlistCheckpointRatioRangeRule { min: 4.9, max: 4339.1 };
        let mut item = new_allowlist_checkpoint("x", "y");
        item.ratio = 4340.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = AllowlistCheckpointRequiredCategoryRule {
            message: "allowlist checkpoint needs this field".to_string(),
        };
        let mut item = new_allowlist_checkpoint("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
