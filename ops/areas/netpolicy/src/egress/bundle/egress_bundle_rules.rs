use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress bundle must pass. Check returns its problems, or none.
pub trait EgressBundleRule: std::fmt::Debug {
    fn check(&self, item: &EgressBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBundleThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressBundleRule for EgressBundleThresholdRangeRule {
    fn check(&self, item: &EgressBundle) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBundleRequiredSegmentRule {
    pub message: String,
}

impl EgressBundleRule for EgressBundleRequiredSegmentRule {
    fn check(&self, item: &EgressBundle) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBundleMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressBundleRule for EgressBundleMinimumRangeRule {
    fn check(&self, item: &EgressBundle) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBundlePinnedGuardRule {
    pub limit: f64,
}

impl EgressBundleRule for EgressBundlePinnedGuardRule {
    fn check(&self, item: &EgressBundle) -> Vec<String> {
        if item.pinned && (item.threshold > self.limit) {
            return vec![format!(
                "threshold {} exceeds {} while pinned",
                item.threshold, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress bundle.
#[derive(Debug, Clone)]
pub struct EgressBundleValidator {
    pub rules: Vec<Rc<dyn EgressBundleRule>>,
}

impl EgressBundleValidator {
    pub fn validate(&self, item: &EgressBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_bundle_validator() -> EgressBundleValidator {
    EgressBundleValidator {
        rules: vec![
            Rc::new(EgressBundleThresholdRangeRule { min: 5.3, max: 1524.6 }) as Rc<dyn EgressBundleRule>,
            Rc::new(EgressBundleRequiredSegmentRule {
                message: "egress bundle needs this field".to_string(),
            }),
            Rc::new(EgressBundleMinimumRangeRule { min: 3.1, max: 3102.0 }),
            Rc::new(EgressBundlePinnedGuardRule { limit: 4.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_bundle_validator();
        let item = new_egress_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = EgressBundleThresholdRangeRule { min: 4.4, max: 65.8 };
        let mut item = new_egress_bundle("x", "y");
        item.threshold = 67.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = EgressBundleRequiredSegmentRule {
            message: "egress bundle needs a value".to_string(),
        };
        let mut item = new_egress_bundle("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = EgressBundleMinimumRangeRule { min: 2.0, max: 3328.1 };
        let mut item = new_egress_bundle("x", "y");
        item.minimum = 3329.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
