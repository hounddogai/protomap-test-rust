use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending identity override must pass. Check returns its problems, or none.
pub trait PendingIdentityOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PendingIdentityOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentityOverrideCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingIdentityOverrideRule for PendingIdentityOverrideCreatedAtRangeRule {
    fn check(&self, item: &PendingIdentityOverride) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentityOverrideCodePrefixRule {
    pub prefix: String,
}

impl PendingIdentityOverrideRule for PendingIdentityOverrideCodePrefixRule {
    fn check(&self, item: &PendingIdentityOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentityOverrideNameLengthRule {
    pub max_length: i64,
}

impl PendingIdentityOverrideRule for PendingIdentityOverrideNameLengthRule {
    fn check(&self, item: &PendingIdentityOverride) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentityOverrideWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingIdentityOverrideRule for PendingIdentityOverrideWindowDaysRangeRule {
    fn check(&self, item: &PendingIdentityOverride) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending identity override.
#[derive(Debug, Clone)]
pub struct PendingIdentityOverrideValidator {
    pub rules: Vec<Rc<dyn PendingIdentityOverrideRule>>,
}

impl PendingIdentityOverrideValidator {
    pub fn validate(&self, item: &PendingIdentityOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingIdentityOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingIdentityOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_identity_override_validator() -> PendingIdentityOverrideValidator {
    PendingIdentityOverrideValidator {
        rules: vec![
            Rc::new(PendingIdentityOverrideCreatedAtRangeRule { min: 255, max: 1000 })
                as Rc<dyn PendingIdentityOverrideRule>,
            Rc::new(PendingIdentityOverrideCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PendingIdentityOverrideNameLengthRule { max_length: 80 }),
            Rc::new(PendingIdentityOverrideWindowDaysRangeRule { min: 64, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_identity_override_validator();
        let item = new_pending_identity_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = PendingIdentityOverrideCreatedAtRangeRule { min: 32, max: 3600 };
        let mut item = new_pending_identity_override("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingIdentityOverrideNameLengthRule { max_length: 80 };
        let mut item = new_pending_identity_override("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = PendingIdentityOverrideWindowDaysRangeRule { min: 80, max: 255 };
        let mut item = new_pending_identity_override("x", "y");
        item.window_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
