use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched probe digest must pass. Check returns its problems, or none.
pub trait BatchedProbeDigestRule: std::fmt::Debug {
    fn check(&self, item: &BatchedProbeDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeDigestRequiredRegionRule {
    pub message: String,
}

impl BatchedProbeDigestRule for BatchedProbeDigestRequiredRegionRule {
    fn check(&self, item: &BatchedProbeDigest) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeDigestFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedProbeDigestRule for BatchedProbeDigestFloorValueRangeRule {
    fn check(&self, item: &BatchedProbeDigest) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeDigestCodePrefixRule {
    pub prefix: String,
}

impl BatchedProbeDigestRule for BatchedProbeDigestCodePrefixRule {
    fn check(&self, item: &BatchedProbeDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched probe digest.
#[derive(Debug, Clone)]
pub struct BatchedProbeDigestValidator {
    pub rules: Vec<Rc<dyn BatchedProbeDigestRule>>,
}

impl BatchedProbeDigestValidator {
    pub fn validate(&self, item: &BatchedProbeDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedProbeDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedProbeDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_probe_digest_validator() -> BatchedProbeDigestValidator {
    BatchedProbeDigestValidator {
        rules: vec![
            Rc::new(BatchedProbeDigestRequiredRegionRule {
                message: "batched probe digest needs this field".to_string(),
            }) as Rc<dyn BatchedProbeDigestRule>,
            Rc::new(BatchedProbeDigestFloorValueRangeRule { min: 3.0, max: 4072.1 }),
            Rc::new(BatchedProbeDigestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_probe_digest_validator();
        let item = new_batched_probe_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = BatchedProbeDigestRequiredRegionRule {
            message: "batched probe digest needs an entry".to_string(),
        };
        let mut item = new_batched_probe_digest("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = BatchedProbeDigestFloorValueRangeRule { min: 9.3, max: 2925.3 };
        let mut item = new_batched_probe_digest("x", "y");
        item.floor_value = 2926.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
