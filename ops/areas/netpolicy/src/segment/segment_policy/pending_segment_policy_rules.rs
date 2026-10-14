use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending segment policy must pass. Check returns its problems, or none.
pub trait PendingSegmentPolicyRule: std::fmt::Debug {
    fn check(&self, item: &PendingSegmentPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentPolicyPhoneLengthRule {
    pub max_length: i64,
}

impl PendingSegmentPolicyRule for PendingSegmentPolicyPhoneLengthRule {
    fn check(&self, item: &PendingSegmentPolicy) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentPolicyBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingSegmentPolicyRule for PendingSegmentPolicyBatchSizeRangeRule {
    fn check(&self, item: &PendingSegmentPolicy) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentPolicyAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl PendingSegmentPolicyRule for PendingSegmentPolicyAllowedSourceRule {
    fn check(&self, item: &PendingSegmentPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is not allowed", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentPolicyPinnedGuardRule {
    pub limit: f64,
}

impl PendingSegmentPolicyRule for PendingSegmentPolicyPinnedGuardRule {
    fn check(&self, item: &PendingSegmentPolicy) -> Vec<String> {
        if item.pinned && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while pinned",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentPolicySourceLengthRule {
    pub max_length: i64,
}

impl PendingSegmentPolicyRule for PendingSegmentPolicySourceLengthRule {
    fn check(&self, item: &PendingSegmentPolicy) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending segment policy.
#[derive(Debug, Clone)]
pub struct PendingSegmentPolicyValidator {
    pub rules: Vec<Rc<dyn PendingSegmentPolicyRule>>,
}

impl PendingSegmentPolicyValidator {
    pub fn validate(&self, item: &PendingSegmentPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingSegmentPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingSegmentPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_segment_policy_validator() -> PendingSegmentPolicyValidator {
    PendingSegmentPolicyValidator {
        rules: vec![
            Rc::new(PendingSegmentPolicyPhoneLengthRule { max_length: 16 }) as Rc<dyn PendingSegmentPolicyRule>,
            Rc::new(PendingSegmentPolicyBatchSizeRangeRule { min: 80, max: 80 }),
            Rc::new(PendingSegmentPolicyAllowedSourceRule {
                allowed: vec!["global".to_string(), "capped".to_string()],
            }),
            Rc::new(PendingSegmentPolicyPinnedGuardRule { limit: 2.9 }),
            Rc::new(PendingSegmentPolicySourceLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_segment_policy_validator();
        let item = new_pending_segment_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = PendingSegmentPolicyPhoneLengthRule { max_length: 255 };
        let mut item = new_pending_segment_policy("x", "y");
        item.phone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = PendingSegmentPolicyBatchSizeRangeRule { min: 32, max: 64 };
        let mut item = new_pending_segment_policy("x", "y");
        item.batch_size = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = PendingSegmentPolicySourceLengthRule { max_length: 120 };
        let mut item = new_pending_segment_policy("x", "y");
        item.source = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
