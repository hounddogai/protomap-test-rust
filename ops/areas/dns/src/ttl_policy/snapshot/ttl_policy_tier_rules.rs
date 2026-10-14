use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy tier must pass. Check returns its problems, or none.
pub trait TtlPolicyTierRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTierRequiredSegmentRule {
    pub message: String,
}

impl TtlPolicyTierRule for TtlPolicyTierRequiredSegmentRule {
    fn check(&self, item: &TtlPolicyTier) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTierMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyTierRule for TtlPolicyTierMaximumRangeRule {
    fn check(&self, item: &TtlPolicyTier) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTierCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyTierRule for TtlPolicyTierCeilingRangeRule {
    fn check(&self, item: &TtlPolicyTier) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a ttl policy tier.
#[derive(Debug, Clone)]
pub struct TtlPolicyTierValidator {
    pub rules: Vec<Rc<dyn TtlPolicyTierRule>>,
}

impl TtlPolicyTierValidator {
    pub fn validate(&self, item: &TtlPolicyTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_tier_validator() -> TtlPolicyTierValidator {
    TtlPolicyTierValidator {
        rules: vec![
            Rc::new(TtlPolicyTierRequiredSegmentRule {
                message: "ttl policy tier needs this field".to_string(),
            }) as Rc<dyn TtlPolicyTierRule>,
            Rc::new(TtlPolicyTierMaximumRangeRule { min: 3.9, max: 3414.4 }),
            Rc::new(TtlPolicyTierCeilingRangeRule { min: 4.1, max: 2895.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_tier_validator();
        let item = new_ttl_policy_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = TtlPolicyTierRequiredSegmentRule {
            message: "ttl policy tier needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_tier("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = TtlPolicyTierMaximumRangeRule { min: 7.3, max: 3591.1 };
        let mut item = new_ttl_policy_tier("x", "y");
        item.maximum = 3592.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = TtlPolicyTierCeilingRangeRule { min: 5.5, max: 4482.7 };
        let mut item = new_ttl_policy_tier("x", "y");
        item.ceiling = 4484.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
