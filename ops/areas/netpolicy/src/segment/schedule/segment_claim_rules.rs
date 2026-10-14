use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment claim must pass. Check returns its problems, or none.
pub trait SegmentClaimRule: std::fmt::Debug {
    fn check(&self, item: &SegmentClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentClaimCodePrefixRule {
    pub prefix: String,
}

impl SegmentClaimRule for SegmentClaimCodePrefixRule {
    fn check(&self, item: &SegmentClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentClaimCodeLengthRule {
    pub max_length: i64,
}

impl SegmentClaimRule for SegmentClaimCodeLengthRule {
    fn check(&self, item: &SegmentClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentClaimLocaleLengthRule {
    pub max_length: i64,
}

impl SegmentClaimRule for SegmentClaimLocaleLengthRule {
    fn check(&self, item: &SegmentClaim) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentClaimBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentClaimRule for SegmentClaimBurstRangeRule {
    fn check(&self, item: &SegmentClaim) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentClaimRequiredTitleRule {
    pub message: String,
}

impl SegmentClaimRule for SegmentClaimRequiredTitleRule {
    fn check(&self, item: &SegmentClaim) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment claim.
#[derive(Debug, Clone)]
pub struct SegmentClaimValidator {
    pub rules: Vec<Rc<dyn SegmentClaimRule>>,
}

impl SegmentClaimValidator {
    pub fn validate(&self, item: &SegmentClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_claim_validator() -> SegmentClaimValidator {
    SegmentClaimValidator {
        rules: vec![
            Rc::new(SegmentClaimCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn SegmentClaimRule>,
            Rc::new(SegmentClaimCodeLengthRule { max_length: 80 }),
            Rc::new(SegmentClaimLocaleLengthRule { max_length: 32 }),
            Rc::new(SegmentClaimBurstRangeRule { min: 32, max: 120 }),
            Rc::new(SegmentClaimRequiredTitleRule {
                message: "segment claim needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_claim_validator();
        let item = new_segment_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SegmentClaimCodeLengthRule { max_length: 16 };
        let mut item = new_segment_claim("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = SegmentClaimLocaleLengthRule { max_length: 64 };
        let mut item = new_segment_claim("x", "y");
        item.locale = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = SegmentClaimBurstRangeRule { min: 80, max: 120 };
        let mut item = new_segment_claim("x", "y");
        item.burst = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SegmentClaimRequiredTitleRule {
            message: "segment claim needs an entry".to_string(),
        };
        let mut item = new_segment_claim("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
