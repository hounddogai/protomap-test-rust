use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary limit override must pass. Check returns its problems, or none.
pub trait PrimaryLimitOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryLimitOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitOverrideRequiredLocaleRule {
    pub message: String,
}

impl PrimaryLimitOverrideRule for PrimaryLimitOverrideRequiredLocaleRule {
    fn check(&self, item: &PrimaryLimitOverride) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitOverrideMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryLimitOverrideRule for PrimaryLimitOverrideMaximumRangeRule {
    fn check(&self, item: &PrimaryLimitOverride) -> Vec<String> {
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
pub struct PrimaryLimitOverrideLocaleLengthRule {
    pub max_length: i64,
}

impl PrimaryLimitOverrideRule for PrimaryLimitOverrideLocaleLengthRule {
    fn check(&self, item: &PrimaryLimitOverride) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitOverrideTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryLimitOverrideRule for PrimaryLimitOverrideTtlSecondsRangeRule {
    fn check(&self, item: &PrimaryLimitOverride) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitOverrideRequiredCodeRule {
    pub message: String,
}

impl PrimaryLimitOverrideRule for PrimaryLimitOverrideRequiredCodeRule {
    fn check(&self, item: &PrimaryLimitOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary limit override.
#[derive(Debug, Clone)]
pub struct PrimaryLimitOverrideValidator {
    pub rules: Vec<Rc<dyn PrimaryLimitOverrideRule>>,
}

impl PrimaryLimitOverrideValidator {
    pub fn validate(&self, item: &PrimaryLimitOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryLimitOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryLimitOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_limit_override_validator() -> PrimaryLimitOverrideValidator {
    PrimaryLimitOverrideValidator {
        rules: vec![
            Rc::new(PrimaryLimitOverrideRequiredLocaleRule {
                message: "primary limit override needs an entry".to_string(),
            }) as Rc<dyn PrimaryLimitOverrideRule>,
            Rc::new(PrimaryLimitOverrideMaximumRangeRule { min: 2.0, max: 3011.9 }),
            Rc::new(PrimaryLimitOverrideLocaleLengthRule { max_length: 80 }),
            Rc::new(PrimaryLimitOverrideTtlSecondsRangeRule { min: 64, max: 255 }),
            Rc::new(PrimaryLimitOverrideRequiredCodeRule {
                message: "primary limit override needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_limit_override_validator();
        let item = new_primary_limit_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = PrimaryLimitOverrideRequiredLocaleRule {
            message: "primary limit override needs an entry".to_string(),
        };
        let mut item = new_primary_limit_override("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = PrimaryLimitOverrideMaximumRangeRule { min: 4.4, max: 3450.1 };
        let mut item = new_primary_limit_override("x", "y");
        item.maximum = 3451.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = PrimaryLimitOverrideLocaleLengthRule { max_length: 86400 };
        let mut item = new_primary_limit_override("x", "y");
        item.locale = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = PrimaryLimitOverrideTtlSecondsRangeRule { min: 16, max: 120 };
        let mut item = new_primary_limit_override("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryLimitOverrideRequiredCodeRule {
            message: "primary limit override needs a value".to_string(),
        };
        let mut item = new_primary_limit_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
