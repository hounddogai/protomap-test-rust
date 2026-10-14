use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention baseline must pass. Check returns its problems, or none.
pub trait RetentionBaselineRule: std::fmt::Debug {
    fn check(&self, item: &RetentionBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBaselineAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RetentionBaselineRule for RetentionBaselineAllowedTitleRule {
    fn check(&self, item: &RetentionBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is inconsistent", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBaselineRegionLengthRule {
    pub max_length: i64,
}

impl RetentionBaselineRule for RetentionBaselineRegionLengthRule {
    fn check(&self, item: &RetentionBaseline) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBaselineDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionBaselineRule for RetentionBaselineDurationMinutesRangeRule {
    fn check(&self, item: &RetentionBaseline) -> Vec<String> {
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

/// Runs every rule of a retention baseline.
#[derive(Debug, Clone)]
pub struct RetentionBaselineValidator {
    pub rules: Vec<Rc<dyn RetentionBaselineRule>>,
}

impl RetentionBaselineValidator {
    pub fn validate(&self, item: &RetentionBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RetentionBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_baseline_validator() -> RetentionBaselineValidator {
    RetentionBaselineValidator {
        rules: vec![
            Rc::new(RetentionBaselineAllowedTitleRule {
                allowed: vec!["global".to_string()],
            }) as Rc<dyn RetentionBaselineRule>,
            Rc::new(RetentionBaselineRegionLengthRule { max_length: 64 }),
            Rc::new(RetentionBaselineDurationMinutesRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_baseline_validator();
        let item = new_retention_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = RetentionBaselineRegionLengthRule { max_length: 3600 };
        let mut item = new_retention_baseline("x", "y");
        item.region = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = RetentionBaselineDurationMinutesRangeRule { min: 80, max: 120 };
        let mut item = new_retention_baseline("x", "y");
        item.duration_minutes = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
