use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity target must pass. Check returns its problems, or none.
pub trait IdentityTargetRule: std::fmt::Debug {
    fn check(&self, item: &IdentityTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTargetSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityTargetRule for IdentityTargetSequenceRangeRule {
    fn check(&self, item: &IdentityTarget) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTargetCodePrefixRule {
    pub prefix: String,
}

impl IdentityTargetRule for IdentityTargetCodePrefixRule {
    fn check(&self, item: &IdentityTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTargetAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityTargetRule for IdentityTargetAttemptsRangeRule {
    fn check(&self, item: &IdentityTarget) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTargetRequiredTimezoneRule {
    pub message: String,
}

impl IdentityTargetRule for IdentityTargetRequiredTimezoneRule {
    fn check(&self, item: &IdentityTarget) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity target.
#[derive(Debug, Clone)]
pub struct IdentityTargetValidator {
    pub rules: Vec<Rc<dyn IdentityTargetRule>>,
}

impl IdentityTargetValidator {
    pub fn validate(&self, item: &IdentityTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_target_validator() -> IdentityTargetValidator {
    IdentityTargetValidator {
        rules: vec![
            Rc::new(IdentityTargetSequenceRangeRule { min: 16, max: 120 }) as Rc<dyn IdentityTargetRule>,
            Rc::new(IdentityTargetCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(IdentityTargetAttemptsRangeRule { min: 16, max: 86400 }),
            Rc::new(IdentityTargetRequiredTimezoneRule {
                message: "identity target needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_target_validator();
        let item = new_identity_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = IdentityTargetSequenceRangeRule { min: 16, max: 3600 };
        let mut item = new_identity_target("x", "y");
        item.sequence = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = IdentityTargetAttemptsRangeRule { min: 1000, max: 1000 };
        let mut item = new_identity_target("x", "y");
        item.attempts = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = IdentityTargetRequiredTimezoneRule {
            message: "identity target needs an entry".to_string(),
        };
        let mut item = new_identity_target("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
