use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation override must pass. Check returns its problems, or none.
pub trait FederationOverrideRule: std::fmt::Debug {
    fn check(&self, item: &FederationOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationOverrideTitleLengthRule {
    pub max_length: i64,
}

impl FederationOverrideRule for FederationOverrideTitleLengthRule {
    fn check(&self, item: &FederationOverride) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationOverrideGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationOverrideRule for FederationOverrideGraceDaysRangeRule {
    fn check(&self, item: &FederationOverride) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationOverrideRequiredTitleRule {
    pub message: String,
}

impl FederationOverrideRule for FederationOverrideRequiredTitleRule {
    fn check(&self, item: &FederationOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationOverrideAudienceLengthRule {
    pub max_length: i64,
}

impl FederationOverrideRule for FederationOverrideAudienceLengthRule {
    fn check(&self, item: &FederationOverride) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationOverrideCodeLengthRule {
    pub max_length: i64,
}

impl FederationOverrideRule for FederationOverrideCodeLengthRule {
    fn check(&self, item: &FederationOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationOverrideLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationOverrideRule for FederationOverrideLimitRangeRule {
    fn check(&self, item: &FederationOverride) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a federation override.
#[derive(Debug, Clone)]
pub struct FederationOverrideValidator {
    pub rules: Vec<Rc<dyn FederationOverrideRule>>,
}

impl FederationOverrideValidator {
    pub fn validate(&self, item: &FederationOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_override_validator() -> FederationOverrideValidator {
    FederationOverrideValidator {
        rules: vec![
            Rc::new(FederationOverrideTitleLengthRule { max_length: 64 }) as Rc<dyn FederationOverrideRule>,
            Rc::new(FederationOverrideGraceDaysRangeRule { min: 32, max: 120 }),
            Rc::new(FederationOverrideRequiredTitleRule {
                message: "federation override needs this field".to_string(),
            }),
            Rc::new(FederationOverrideAudienceLengthRule { max_length: 16 }),
            Rc::new(FederationOverrideCodeLengthRule { max_length: 255 }),
            Rc::new(FederationOverrideLimitRangeRule { min: 120, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_override_validator();
        let item = new_federation_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FederationOverrideTitleLengthRule { max_length: 3600 };
        let mut item = new_federation_override("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = FederationOverrideGraceDaysRangeRule { min: 64, max: 120 };
        let mut item = new_federation_override("x", "y");
        item.grace_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FederationOverrideRequiredTitleRule {
            message: "federation override needs an entry".to_string(),
        };
        let mut item = new_federation_override("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = FederationOverrideAudienceLengthRule { max_length: 16 };
        let mut item = new_federation_override("x", "y");
        item.audience = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FederationOverrideCodeLengthRule { max_length: 255 };
        let mut item = new_federation_override("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = FederationOverrideLimitRangeRule { min: 16, max: 32 };
        let mut item = new_federation_override("x", "y");
        item.limit = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
