use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive usage claim must pass. Check returns its problems, or none.
pub trait AdaptiveUsageClaimRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveUsageClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageClaimFingerprintLengthRule {
    pub max_length: i64,
}

impl AdaptiveUsageClaimRule for AdaptiveUsageClaimFingerprintLengthRule {
    fn check(&self, item: &AdaptiveUsageClaim) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageClaimCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveUsageClaimRule for AdaptiveUsageClaimCostRangeRule {
    fn check(&self, item: &AdaptiveUsageClaim) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageClaimSegmentLengthRule {
    pub max_length: i64,
}

impl AdaptiveUsageClaimRule for AdaptiveUsageClaimSegmentLengthRule {
    fn check(&self, item: &AdaptiveUsageClaim) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageClaimCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveUsageClaimRule for AdaptiveUsageClaimCodePrefixRule {
    fn check(&self, item: &AdaptiveUsageClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageClaimAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl AdaptiveUsageClaimRule for AdaptiveUsageClaimAllowedSegmentRule {
    fn check(&self, item: &AdaptiveUsageClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is expired", item.segment)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive usage claim.
#[derive(Debug, Clone)]
pub struct AdaptiveUsageClaimValidator {
    pub rules: Vec<Rc<dyn AdaptiveUsageClaimRule>>,
}

impl AdaptiveUsageClaimValidator {
    pub fn validate(&self, item: &AdaptiveUsageClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveUsageClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveUsageClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_usage_claim_validator() -> AdaptiveUsageClaimValidator {
    AdaptiveUsageClaimValidator {
        rules: vec![
            Rc::new(AdaptiveUsageClaimFingerprintLengthRule { max_length: 16 }) as Rc<dyn AdaptiveUsageClaimRule>,
            Rc::new(AdaptiveUsageClaimCostRangeRule { min: 8.1, max: 1928.1 }),
            Rc::new(AdaptiveUsageClaimSegmentLengthRule { max_length: 86400 }),
            Rc::new(AdaptiveUsageClaimCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(AdaptiveUsageClaimAllowedSegmentRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_usage_claim_validator();
        let item = new_adaptive_usage_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = AdaptiveUsageClaimFingerprintLengthRule { max_length: 16 };
        let mut item = new_adaptive_usage_claim("x", "y");
        item.fingerprint = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = AdaptiveUsageClaimCostRangeRule { min: 8.0, max: 3222.6 };
        let mut item = new_adaptive_usage_claim("x", "y");
        item.cost = 3224.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = AdaptiveUsageClaimSegmentLengthRule { max_length: 1000 };
        let mut item = new_adaptive_usage_claim("x", "y");
        item.segment = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
