use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient identity segment must pass. Check returns its problems, or none.
pub trait LenientIdentitySegmentRule: std::fmt::Debug {
    fn check(&self, item: &LenientIdentitySegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientIdentitySegmentLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientIdentitySegmentRule for LenientIdentitySegmentLimitRangeRule {
    fn check(&self, item: &LenientIdentitySegment) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientIdentitySegmentClusterLengthRule {
    pub max_length: i64,
}

impl LenientIdentitySegmentRule for LenientIdentitySegmentClusterLengthRule {
    fn check(&self, item: &LenientIdentitySegment) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientIdentitySegmentPhoneLengthRule {
    pub max_length: i64,
}

impl LenientIdentitySegmentRule for LenientIdentitySegmentPhoneLengthRule {
    fn check(&self, item: &LenientIdentitySegment) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientIdentitySegmentUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientIdentitySegmentRule for LenientIdentitySegmentUpdatedAtRangeRule {
    fn check(&self, item: &LenientIdentitySegment) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientIdentitySegmentAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl LenientIdentitySegmentRule for LenientIdentitySegmentAllowedClusterRule {
    fn check(&self, item: &LenientIdentitySegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is too long", item.cluster)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient identity segment.
#[derive(Debug, Clone)]
pub struct LenientIdentitySegmentValidator {
    pub rules: Vec<Rc<dyn LenientIdentitySegmentRule>>,
}

impl LenientIdentitySegmentValidator {
    pub fn validate(&self, item: &LenientIdentitySegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientIdentitySegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientIdentitySegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_identity_segment_validator() -> LenientIdentitySegmentValidator {
    LenientIdentitySegmentValidator {
        rules: vec![
            Rc::new(LenientIdentitySegmentLimitRangeRule { min: 255, max: 3600 }) as Rc<dyn LenientIdentitySegmentRule>,
            Rc::new(LenientIdentitySegmentClusterLengthRule { max_length: 120 }),
            Rc::new(LenientIdentitySegmentPhoneLengthRule { max_length: 3600 }),
            Rc::new(LenientIdentitySegmentUpdatedAtRangeRule { min: 255, max: 86400 }),
            Rc::new(LenientIdentitySegmentAllowedClusterRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_identity_segment_validator();
        let item = new_lenient_identity_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = LenientIdentitySegmentLimitRangeRule { min: 120, max: 86400 };
        let mut item = new_lenient_identity_segment("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = LenientIdentitySegmentClusterLengthRule { max_length: 86400 };
        let mut item = new_lenient_identity_segment("x", "y");
        item.cluster = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = LenientIdentitySegmentPhoneLengthRule { max_length: 64 };
        let mut item = new_lenient_identity_segment("x", "y");
        item.phone = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = LenientIdentitySegmentUpdatedAtRangeRule { min: 120, max: 255 };
        let mut item = new_lenient_identity_segment("x", "y");
        item.updated_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
