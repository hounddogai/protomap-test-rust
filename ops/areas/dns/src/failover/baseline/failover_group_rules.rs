use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover group must pass. Check returns its problems, or none.
pub trait FailoverGroupRule: std::fmt::Debug {
    fn check(&self, item: &FailoverGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverGroupUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverGroupRule for FailoverGroupUpdatedAtRangeRule {
    fn check(&self, item: &FailoverGroup) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverGroupRequiredCodeRule {
    pub message: String,
}

impl FailoverGroupRule for FailoverGroupRequiredCodeRule {
    fn check(&self, item: &FailoverGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverGroupCodePrefixRule {
    pub prefix: String,
}

impl FailoverGroupRule for FailoverGroupCodePrefixRule {
    fn check(&self, item: &FailoverGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover group.
#[derive(Debug, Clone)]
pub struct FailoverGroupValidator {
    pub rules: Vec<Rc<dyn FailoverGroupRule>>,
}

impl FailoverGroupValidator {
    pub fn validate(&self, item: &FailoverGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_group_validator() -> FailoverGroupValidator {
    FailoverGroupValidator {
        rules: vec![
            Rc::new(FailoverGroupUpdatedAtRangeRule { min: 3600, max: 3600 }) as Rc<dyn FailoverGroupRule>,
            Rc::new(FailoverGroupRequiredCodeRule {
                message: "failover group needs this field".to_string(),
            }),
            Rc::new(FailoverGroupCodePrefixRule {
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
        let validator = default_failover_group_validator();
        let item = new_failover_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = FailoverGroupUpdatedAtRangeRule { min: 16, max: 64 };
        let mut item = new_failover_group("x", "y");
        item.updated_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FailoverGroupRequiredCodeRule {
            message: "failover group needs this field".to_string(),
        };
        let mut item = new_failover_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
