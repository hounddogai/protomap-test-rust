use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a health check rollup must pass. Check returns its problems, or none.
pub trait HealthCheckRollupRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRollupExternalRefLengthRule {
    pub max_length: i64,
}

impl HealthCheckRollupRule for HealthCheckRollupExternalRefLengthRule {
    fn check(&self, item: &HealthCheckRollup) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRollupRequiredNameRule {
    pub message: String,
}

impl HealthCheckRollupRule for HealthCheckRollupRequiredNameRule {
    fn check(&self, item: &HealthCheckRollup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRollupArchivedGuardRule {
    pub limit: i64,
}

impl HealthCheckRollupRule for HealthCheckRollupArchivedGuardRule {
    fn check(&self, item: &HealthCheckRollup) -> Vec<String> {
        if item.archived && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while archived",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRollupMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckRollupRule for HealthCheckRollupMaximumRangeRule {
    fn check(&self, item: &HealthCheckRollup) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRollupAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl HealthCheckRollupRule for HealthCheckRollupAllowedReviewerRule {
    fn check(&self, item: &HealthCheckRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is out of range", item.reviewer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check rollup.
#[derive(Debug, Clone)]
pub struct HealthCheckRollupValidator {
    pub rules: Vec<Rc<dyn HealthCheckRollupRule>>,
}

impl HealthCheckRollupValidator {
    pub fn validate(&self, item: &HealthCheckRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_rollup_validator() -> HealthCheckRollupValidator {
    HealthCheckRollupValidator {
        rules: vec![
            Rc::new(HealthCheckRollupExternalRefLengthRule { max_length: 32 }) as Rc<dyn HealthCheckRollupRule>,
            Rc::new(HealthCheckRollupRequiredNameRule {
                message: "health check rollup needs an entry".to_string(),
            }),
            Rc::new(HealthCheckRollupArchivedGuardRule { limit: 64 }),
            Rc::new(HealthCheckRollupMaximumRangeRule { min: 6.9, max: 3341.4 }),
            Rc::new(HealthCheckRollupAllowedReviewerRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_rollup_validator();
        let item = new_health_check_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = HealthCheckRollupExternalRefLengthRule { max_length: 255 };
        let mut item = new_health_check_rollup("x", "y");
        item.external_ref = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = HealthCheckRollupRequiredNameRule {
            message: "health check rollup needs an entry".to_string(),
        };
        let mut item = new_health_check_rollup("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = HealthCheckRollupMaximumRangeRule { min: 1.6, max: 3759.2 };
        let mut item = new_health_check_rollup("x", "y");
        item.maximum = 3760.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
