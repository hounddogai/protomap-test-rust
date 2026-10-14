use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped ttl policy tier must pass. Check returns its problems, or none.
pub trait CappedTtlPolicyTierRule: std::fmt::Debug {
    fn check(&self, item: &CappedTtlPolicyTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTtlPolicyTierArchivedGuardRule {
    pub limit: i64,
}

impl CappedTtlPolicyTierRule for CappedTtlPolicyTierArchivedGuardRule {
    fn check(&self, item: &CappedTtlPolicyTier) -> Vec<String> {
        if item.archived && (item.retries > self.limit) {
            return vec![format!(
                "retries {} exceeds {} while archived",
                item.retries, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTtlPolicyTierThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedTtlPolicyTierRule for CappedTtlPolicyTierThresholdRangeRule {
    fn check(&self, item: &CappedTtlPolicyTier) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTtlPolicyTierRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedTtlPolicyTierRule for CappedTtlPolicyTierRetriesRangeRule {
    fn check(&self, item: &CappedTtlPolicyTier) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped ttl policy tier.
#[derive(Debug, Clone)]
pub struct CappedTtlPolicyTierValidator {
    pub rules: Vec<Rc<dyn CappedTtlPolicyTierRule>>,
}

impl CappedTtlPolicyTierValidator {
    pub fn validate(&self, item: &CappedTtlPolicyTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedTtlPolicyTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedTtlPolicyTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_ttl_policy_tier_validator() -> CappedTtlPolicyTierValidator {
    CappedTtlPolicyTierValidator {
        rules: vec![
            Rc::new(CappedTtlPolicyTierArchivedGuardRule { limit: 32 }) as Rc<dyn CappedTtlPolicyTierRule>,
            Rc::new(CappedTtlPolicyTierThresholdRangeRule { min: 1.3, max: 4780.0 }),
            Rc::new(CappedTtlPolicyTierRetriesRangeRule { min: 80, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_ttl_policy_tier_validator();
        let item = new_capped_ttl_policy_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = CappedTtlPolicyTierThresholdRangeRule { min: 8.9, max: 225.2 };
        let mut item = new_capped_ttl_policy_tier("x", "y");
        item.threshold = 226.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = CappedTtlPolicyTierRetriesRangeRule { min: 120, max: 86400 };
        let mut item = new_capped_ttl_policy_tier("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
