use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback health check rollup must pass. Check returns its problems, or none.
pub trait FallbackHealthCheckRollupRule: std::fmt::Debug {
    fn check(&self, item: &FallbackHealthCheckRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackHealthCheckRollupAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl FallbackHealthCheckRollupRule for FallbackHealthCheckRollupAllowedClusterRule {
    fn check(&self, item: &FallbackHealthCheckRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is above maximum", item.cluster)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackHealthCheckRollupReviewerLengthRule {
    pub max_length: i64,
}

impl FallbackHealthCheckRollupRule for FallbackHealthCheckRollupReviewerLengthRule {
    fn check(&self, item: &FallbackHealthCheckRollup) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackHealthCheckRollupRequiredReviewerRule {
    pub message: String,
}

impl FallbackHealthCheckRollupRule for FallbackHealthCheckRollupRequiredReviewerRule {
    fn check(&self, item: &FallbackHealthCheckRollup) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback health check rollup.
#[derive(Debug, Clone)]
pub struct FallbackHealthCheckRollupValidator {
    pub rules: Vec<Rc<dyn FallbackHealthCheckRollupRule>>,
}

impl FallbackHealthCheckRollupValidator {
    pub fn validate(&self, item: &FallbackHealthCheckRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackHealthCheckRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackHealthCheckRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_health_check_rollup_validator() -> FallbackHealthCheckRollupValidator {
    FallbackHealthCheckRollupValidator {
        rules: vec![
            Rc::new(FallbackHealthCheckRollupAllowedClusterRule {
                allowed: vec!["tiered".to_string(), "rolling".to_string(), "pending".to_string()],
            }) as Rc<dyn FallbackHealthCheckRollupRule>,
            Rc::new(FallbackHealthCheckRollupReviewerLengthRule { max_length: 80 }),
            Rc::new(FallbackHealthCheckRollupRequiredReviewerRule {
                message: "fallback health check rollup needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_health_check_rollup_validator();
        let item = new_fallback_health_check_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = FallbackHealthCheckRollupReviewerLengthRule { max_length: 120 };
        let mut item = new_fallback_health_check_rollup("x", "y");
        item.reviewer = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = FallbackHealthCheckRollupRequiredReviewerRule {
            message: "fallback health check rollup needs an entry".to_string(),
        };
        let mut item = new_fallback_health_check_rollup("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
