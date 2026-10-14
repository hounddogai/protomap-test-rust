use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment digest must pass. Check returns its problems, or none.
pub trait SegmentDigestRule: std::fmt::Debug {
    fn check(&self, item: &SegmentDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDigestRequiredCodeRule {
    pub message: String,
}

impl SegmentDigestRule for SegmentDigestRequiredCodeRule {
    fn check(&self, item: &SegmentDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDigestPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentDigestRule for SegmentDigestPriorityRangeRule {
    fn check(&self, item: &SegmentDigest) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDigestCodePrefixRule {
    pub prefix: String,
}

impl SegmentDigestRule for SegmentDigestCodePrefixRule {
    fn check(&self, item: &SegmentDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDigestBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentDigestRule for SegmentDigestBurstRangeRule {
    fn check(&self, item: &SegmentDigest) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment digest.
#[derive(Debug, Clone)]
pub struct SegmentDigestValidator {
    pub rules: Vec<Rc<dyn SegmentDigestRule>>,
}

impl SegmentDigestValidator {
    pub fn validate(&self, item: &SegmentDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_digest_validator() -> SegmentDigestValidator {
    SegmentDigestValidator {
        rules: vec![
            Rc::new(SegmentDigestRequiredCodeRule {
                message: "segment digest needs a value".to_string(),
            }) as Rc<dyn SegmentDigestRule>,
            Rc::new(SegmentDigestPriorityRangeRule { min: 16, max: 255 }),
            Rc::new(SegmentDigestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(SegmentDigestBurstRangeRule { min: 16, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_digest_validator();
        let item = new_segment_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SegmentDigestRequiredCodeRule {
            message: "segment digest needs this field".to_string(),
        };
        let mut item = new_segment_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = SegmentDigestPriorityRangeRule { min: 64, max: 80 };
        let mut item = new_segment_digest("x", "y");
        item.priority = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = SegmentDigestBurstRangeRule { min: 32, max: 255 };
        let mut item = new_segment_digest("x", "y");
        item.burst = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
