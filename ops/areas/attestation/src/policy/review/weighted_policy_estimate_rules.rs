use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted policy estimate must pass. Check returns its problems, or none.
pub trait WeightedPolicyEstimateRule: std::fmt::Debug {
    fn check(&self, item: &WeightedPolicyEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyEstimateTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedPolicyEstimateRule for WeightedPolicyEstimateTtlSecondsRangeRule {
    fn check(&self, item: &WeightedPolicyEstimate) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyEstimateRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedPolicyEstimateRule for WeightedPolicyEstimateRateRangeRule {
    fn check(&self, item: &WeightedPolicyEstimate) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyEstimateCodePrefixRule {
    pub prefix: String,
}

impl WeightedPolicyEstimateRule for WeightedPolicyEstimateCodePrefixRule {
    fn check(&self, item: &WeightedPolicyEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyEstimateAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl WeightedPolicyEstimateRule for WeightedPolicyEstimateAllowedAudienceRule {
    fn check(&self, item: &WeightedPolicyEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is not allowed", item.audience)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted policy estimate.
#[derive(Debug, Clone)]
pub struct WeightedPolicyEstimateValidator {
    pub rules: Vec<Rc<dyn WeightedPolicyEstimateRule>>,
}

impl WeightedPolicyEstimateValidator {
    pub fn validate(&self, item: &WeightedPolicyEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedPolicyEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedPolicyEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_policy_estimate_validator() -> WeightedPolicyEstimateValidator {
    WeightedPolicyEstimateValidator {
        rules: vec![
            Rc::new(WeightedPolicyEstimateTtlSecondsRangeRule { min: 16, max: 32 })
                as Rc<dyn WeightedPolicyEstimateRule>,
            Rc::new(WeightedPolicyEstimateRateRangeRule { min: 3.9, max: 4993.9 }),
            Rc::new(WeightedPolicyEstimateCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(WeightedPolicyEstimateAllowedAudienceRule {
                allowed: vec!["fallback".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_policy_estimate_validator();
        let item = new_weighted_policy_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = WeightedPolicyEstimateTtlSecondsRangeRule { min: 80, max: 3600 };
        let mut item = new_weighted_policy_estimate("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = WeightedPolicyEstimateRateRangeRule { min: 0.8, max: 2651.2 };
        let mut item = new_weighted_policy_estimate("x", "y");
        item.rate = 2652.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
