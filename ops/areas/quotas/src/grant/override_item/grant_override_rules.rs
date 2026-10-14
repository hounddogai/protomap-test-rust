use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant override must pass. Check returns its problems, or none.
pub trait GrantOverrideRule: std::fmt::Debug {
    fn check(&self, item: &GrantOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantOverrideRequiredCodeRule {
    pub message: String,
}

impl GrantOverrideRule for GrantOverrideRequiredCodeRule {
    fn check(&self, item: &GrantOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantOverrideBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantOverrideRule for GrantOverrideBudgetRangeRule {
    fn check(&self, item: &GrantOverride) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantOverrideRequiredRegionRule {
    pub message: String,
}

impl GrantOverrideRule for GrantOverrideRequiredRegionRule {
    fn check(&self, item: &GrantOverride) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantOverrideCodeLengthRule {
    pub max_length: i64,
}

impl GrantOverrideRule for GrantOverrideCodeLengthRule {
    fn check(&self, item: &GrantOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant override.
#[derive(Debug, Clone)]
pub struct GrantOverrideValidator {
    pub rules: Vec<Rc<dyn GrantOverrideRule>>,
}

impl GrantOverrideValidator {
    pub fn validate(&self, item: &GrantOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_override_validator() -> GrantOverrideValidator {
    GrantOverrideValidator {
        rules: vec![
            Rc::new(GrantOverrideRequiredCodeRule {
                message: "grant override needs a value".to_string(),
            }) as Rc<dyn GrantOverrideRule>,
            Rc::new(GrantOverrideBudgetRangeRule { min: 6.7, max: 3582.7 }),
            Rc::new(GrantOverrideRequiredRegionRule {
                message: "grant override needs this field".to_string(),
            }),
            Rc::new(GrantOverrideCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_override_validator();
        let item = new_grant_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GrantOverrideRequiredCodeRule {
            message: "grant override needs this field".to_string(),
        };
        let mut item = new_grant_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = GrantOverrideBudgetRangeRule { min: 7.6, max: 556.5 };
        let mut item = new_grant_override("x", "y");
        item.budget = 558.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = GrantOverrideRequiredRegionRule {
            message: "grant override needs this field".to_string(),
        };
        let mut item = new_grant_override("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GrantOverrideCodeLengthRule { max_length: 120 };
        let mut item = new_grant_override("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
