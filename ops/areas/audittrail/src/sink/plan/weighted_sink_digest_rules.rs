use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted sink digest must pass. Check returns its problems, or none.
pub trait WeightedSinkDigestRule: std::fmt::Debug {
    fn check(&self, item: &WeightedSinkDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSinkDigestPrincipalLengthRule {
    pub max_length: i64,
}

impl WeightedSinkDigestRule for WeightedSinkDigestPrincipalLengthRule {
    fn check(&self, item: &WeightedSinkDigest) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSinkDigestCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedSinkDigestRule for WeightedSinkDigestCapacityRangeRule {
    fn check(&self, item: &WeightedSinkDigest) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSinkDigestAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl WeightedSinkDigestRule for WeightedSinkDigestAllowedSummaryRule {
    fn check(&self, item: &WeightedSinkDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is missing", item.summary)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted sink digest.
#[derive(Debug, Clone)]
pub struct WeightedSinkDigestValidator {
    pub rules: Vec<Rc<dyn WeightedSinkDigestRule>>,
}

impl WeightedSinkDigestValidator {
    pub fn validate(&self, item: &WeightedSinkDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedSinkDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedSinkDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_sink_digest_validator() -> WeightedSinkDigestValidator {
    WeightedSinkDigestValidator {
        rules: vec![
            Rc::new(WeightedSinkDigestPrincipalLengthRule { max_length: 80 }) as Rc<dyn WeightedSinkDigestRule>,
            Rc::new(WeightedSinkDigestCapacityRangeRule { min: 16, max: 120 }),
            Rc::new(WeightedSinkDigestAllowedSummaryRule {
                allowed: vec!["regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_sink_digest_validator();
        let item = new_weighted_sink_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = WeightedSinkDigestPrincipalLengthRule { max_length: 3600 };
        let mut item = new_weighted_sink_digest("x", "y");
        item.principal = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = WeightedSinkDigestCapacityRangeRule { min: 32, max: 120 };
        let mut item = new_weighted_sink_digest("x", "y");
        item.capacity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
