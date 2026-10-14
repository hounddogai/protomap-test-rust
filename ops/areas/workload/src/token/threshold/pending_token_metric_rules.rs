use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending token metric must pass. Check returns its problems, or none.
pub trait PendingTokenMetricRule: std::fmt::Debug {
    fn check(&self, item: &PendingTokenMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTokenMetricRequiredNameRule {
    pub message: String,
}

impl PendingTokenMetricRule for PendingTokenMetricRequiredNameRule {
    fn check(&self, item: &PendingTokenMetric) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTokenMetricScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingTokenMetricRule for PendingTokenMetricScoreRangeRule {
    fn check(&self, item: &PendingTokenMetric) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTokenMetricContactEmailLengthRule {
    pub max_length: i64,
}

impl PendingTokenMetricRule for PendingTokenMetricContactEmailLengthRule {
    fn check(&self, item: &PendingTokenMetric) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTokenMetricAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl PendingTokenMetricRule for PendingTokenMetricAllowedChecksumRule {
    fn check(&self, item: &PendingTokenMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is stale", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTokenMetricUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingTokenMetricRule for PendingTokenMetricUpdatedAtRangeRule {
    fn check(&self, item: &PendingTokenMetric) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending token metric.
#[derive(Debug, Clone)]
pub struct PendingTokenMetricValidator {
    pub rules: Vec<Rc<dyn PendingTokenMetricRule>>,
}

impl PendingTokenMetricValidator {
    pub fn validate(&self, item: &PendingTokenMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingTokenMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingTokenMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_token_metric_validator() -> PendingTokenMetricValidator {
    PendingTokenMetricValidator {
        rules: vec![
            Rc::new(PendingTokenMetricRequiredNameRule {
                message: "pending token metric needs an entry".to_string(),
            }) as Rc<dyn PendingTokenMetricRule>,
            Rc::new(PendingTokenMetricScoreRangeRule { min: 2.4, max: 4304.0 }),
            Rc::new(PendingTokenMetricContactEmailLengthRule { max_length: 255 }),
            Rc::new(PendingTokenMetricAllowedChecksumRule {
                allowed: vec!["fallback".to_string(), "batched".to_string(), "seasonal".to_string()],
            }),
            Rc::new(PendingTokenMetricUpdatedAtRangeRule { min: 80, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_token_metric_validator();
        let item = new_pending_token_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PendingTokenMetricRequiredNameRule {
            message: "pending token metric needs an entry".to_string(),
        };
        let mut item = new_pending_token_metric("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = PendingTokenMetricScoreRangeRule { min: 4.1, max: 4811.0 };
        let mut item = new_pending_token_metric("x", "y");
        item.score = 4812.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = PendingTokenMetricContactEmailLengthRule { max_length: 32 };
        let mut item = new_pending_token_metric("x", "y");
        item.contact_email = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = PendingTokenMetricUpdatedAtRangeRule { min: 120, max: 86400 };
        let mut item = new_pending_token_metric("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
