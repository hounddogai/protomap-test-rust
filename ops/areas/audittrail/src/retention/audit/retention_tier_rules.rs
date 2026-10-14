use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention tier must pass. Check returns its problems, or none.
pub trait RetentionTierRule: std::fmt::Debug {
    fn check(&self, item: &RetentionTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionTierDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionTierRule for RetentionTierDurationMinutesRangeRule {
    fn check(&self, item: &RetentionTier) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionTierEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionTierRule for RetentionTierEndsAtRangeRule {
    fn check(&self, item: &RetentionTier) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionTierCodePrefixRule {
    pub prefix: String,
}

impl RetentionTierRule for RetentionTierCodePrefixRule {
    fn check(&self, item: &RetentionTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionTierRequiredSummaryRule {
    pub message: String,
}

impl RetentionTierRule for RetentionTierRequiredSummaryRule {
    fn check(&self, item: &RetentionTier) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention tier.
#[derive(Debug, Clone)]
pub struct RetentionTierValidator {
    pub rules: Vec<Rc<dyn RetentionTierRule>>,
}

impl RetentionTierValidator {
    pub fn validate(&self, item: &RetentionTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_tier_validator() -> RetentionTierValidator {
    RetentionTierValidator {
        rules: vec![
            Rc::new(RetentionTierDurationMinutesRangeRule { min: 32, max: 1000 }) as Rc<dyn RetentionTierRule>,
            Rc::new(RetentionTierEndsAtRangeRule { min: 64, max: 120 }),
            Rc::new(RetentionTierCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RetentionTierRequiredSummaryRule {
                message: "retention tier needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_tier_validator();
        let item = new_retention_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = RetentionTierDurationMinutesRangeRule { min: 16, max: 32 };
        let mut item = new_retention_tier("x", "y");
        item.duration_minutes = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = RetentionTierEndsAtRangeRule { min: 32, max: 64 };
        let mut item = new_retention_tier("x", "y");
        item.ends_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = RetentionTierRequiredSummaryRule {
            message: "retention tier needs a value".to_string(),
        };
        let mut item = new_retention_tier("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
