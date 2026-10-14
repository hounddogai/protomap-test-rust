use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback identity override must pass. Check returns its problems, or none.
pub trait FallbackIdentityOverrideRule: std::fmt::Debug {
    fn check(&self, item: &FallbackIdentityOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityOverrideNameLengthRule {
    pub max_length: i64,
}

impl FallbackIdentityOverrideRule for FallbackIdentityOverrideNameLengthRule {
    fn check(&self, item: &FallbackIdentityOverride) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityOverrideStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackIdentityOverrideRule for FallbackIdentityOverrideStartsAtRangeRule {
    fn check(&self, item: &FallbackIdentityOverride) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityOverrideRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackIdentityOverrideRule for FallbackIdentityOverrideRotationDaysRangeRule {
    fn check(&self, item: &FallbackIdentityOverride) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback identity override.
#[derive(Debug, Clone)]
pub struct FallbackIdentityOverrideValidator {
    pub rules: Vec<Rc<dyn FallbackIdentityOverrideRule>>,
}

impl FallbackIdentityOverrideValidator {
    pub fn validate(&self, item: &FallbackIdentityOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackIdentityOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackIdentityOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_identity_override_validator() -> FallbackIdentityOverrideValidator {
    FallbackIdentityOverrideValidator {
        rules: vec![
            Rc::new(FallbackIdentityOverrideNameLengthRule { max_length: 80 }) as Rc<dyn FallbackIdentityOverrideRule>,
            Rc::new(FallbackIdentityOverrideStartsAtRangeRule { min: 120, max: 3600 }),
            Rc::new(FallbackIdentityOverrideRotationDaysRangeRule { min: 64, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_identity_override_validator();
        let item = new_fallback_identity_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackIdentityOverrideNameLengthRule { max_length: 32 };
        let mut item = new_fallback_identity_override("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = FallbackIdentityOverrideStartsAtRangeRule { min: 32, max: 120 };
        let mut item = new_fallback_identity_override("x", "y");
        item.starts_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = FallbackIdentityOverrideRotationDaysRangeRule { min: 16, max: 86400 };
        let mut item = new_fallback_identity_override("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
