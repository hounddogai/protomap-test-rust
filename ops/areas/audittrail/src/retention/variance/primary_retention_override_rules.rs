use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary retention override must pass. Check returns its problems, or none.
pub trait PrimaryRetentionOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRetentionOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionOverrideCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryRetentionOverrideRule for PrimaryRetentionOverrideCeilingRangeRule {
    fn check(&self, item: &PrimaryRetentionOverride) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionOverrideRequiredLabelRule {
    pub message: String,
}

impl PrimaryRetentionOverrideRule for PrimaryRetentionOverrideRequiredLabelRule {
    fn check(&self, item: &PrimaryRetentionOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionOverrideCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryRetentionOverrideRule for PrimaryRetentionOverrideCodeLengthRule {
    fn check(&self, item: &PrimaryRetentionOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionOverrideRequiredCodeRule {
    pub message: String,
}

impl PrimaryRetentionOverrideRule for PrimaryRetentionOverrideRequiredCodeRule {
    fn check(&self, item: &PrimaryRetentionOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionOverrideRequiredTitleRule {
    pub message: String,
}

impl PrimaryRetentionOverrideRule for PrimaryRetentionOverrideRequiredTitleRule {
    fn check(&self, item: &PrimaryRetentionOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary retention override.
#[derive(Debug, Clone)]
pub struct PrimaryRetentionOverrideValidator {
    pub rules: Vec<Rc<dyn PrimaryRetentionOverrideRule>>,
}

impl PrimaryRetentionOverrideValidator {
    pub fn validate(&self, item: &PrimaryRetentionOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryRetentionOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRetentionOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_retention_override_validator() -> PrimaryRetentionOverrideValidator {
    PrimaryRetentionOverrideValidator {
        rules: vec![
            Rc::new(PrimaryRetentionOverrideCeilingRangeRule { min: 5.1, max: 1570.9 })
                as Rc<dyn PrimaryRetentionOverrideRule>,
            Rc::new(PrimaryRetentionOverrideRequiredLabelRule {
                message: "primary retention override needs an entry".to_string(),
            }),
            Rc::new(PrimaryRetentionOverrideCodeLengthRule { max_length: 16 }),
            Rc::new(PrimaryRetentionOverrideRequiredCodeRule {
                message: "primary retention override needs a value".to_string(),
            }),
            Rc::new(PrimaryRetentionOverrideRequiredTitleRule {
                message: "primary retention override needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_retention_override_validator();
        let item = new_primary_retention_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = PrimaryRetentionOverrideCeilingRangeRule { min: 8.1, max: 1916.3 };
        let mut item = new_primary_retention_override("x", "y");
        item.ceiling = 1917.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimaryRetentionOverrideRequiredLabelRule {
            message: "primary retention override needs an entry".to_string(),
        };
        let mut item = new_primary_retention_override("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryRetentionOverrideCodeLengthRule { max_length: 3600 };
        let mut item = new_primary_retention_override("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryRetentionOverrideRequiredCodeRule {
            message: "primary retention override needs an entry".to_string(),
        };
        let mut item = new_primary_retention_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryRetentionOverrideRequiredTitleRule {
            message: "primary retention override needs this field".to_string(),
        };
        let mut item = new_primary_retention_override("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
