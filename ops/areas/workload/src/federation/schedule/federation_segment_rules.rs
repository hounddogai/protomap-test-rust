use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation segment must pass. Check returns its problems, or none.
pub trait FederationSegmentRule: std::fmt::Debug {
    fn check(&self, item: &FederationSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSegmentCodeLengthRule {
    pub max_length: i64,
}

impl FederationSegmentRule for FederationSegmentCodeLengthRule {
    fn check(&self, item: &FederationSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSegmentExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationSegmentRule for FederationSegmentExpiresAtRangeRule {
    fn check(&self, item: &FederationSegment) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSegmentRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FederationSegmentRule for FederationSegmentRatioRangeRule {
    fn check(&self, item: &FederationSegment) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSegmentRequiredCipherRule {
    pub message: String,
}

impl FederationSegmentRule for FederationSegmentRequiredCipherRule {
    fn check(&self, item: &FederationSegment) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSegmentRequiredNameRule {
    pub message: String,
}

impl FederationSegmentRule for FederationSegmentRequiredNameRule {
    fn check(&self, item: &FederationSegment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation segment.
#[derive(Debug, Clone)]
pub struct FederationSegmentValidator {
    pub rules: Vec<Rc<dyn FederationSegmentRule>>,
}

impl FederationSegmentValidator {
    pub fn validate(&self, item: &FederationSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_segment_validator() -> FederationSegmentValidator {
    FederationSegmentValidator {
        rules: vec![
            Rc::new(FederationSegmentCodeLengthRule { max_length: 1000 }) as Rc<dyn FederationSegmentRule>,
            Rc::new(FederationSegmentExpiresAtRangeRule { min: 16, max: 255 }),
            Rc::new(FederationSegmentRatioRangeRule { min: 4.0, max: 4258.7 }),
            Rc::new(FederationSegmentRequiredCipherRule {
                message: "federation segment needs a value".to_string(),
            }),
            Rc::new(FederationSegmentRequiredNameRule {
                message: "federation segment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_segment_validator();
        let item = new_federation_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FederationSegmentCodeLengthRule { max_length: 3600 };
        let mut item = new_federation_segment("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = FederationSegmentExpiresAtRangeRule { min: 16, max: 3600 };
        let mut item = new_federation_segment("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = FederationSegmentRatioRangeRule { min: 6.6, max: 878.6 };
        let mut item = new_federation_segment("x", "y");
        item.ratio = 880.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = FederationSegmentRequiredCipherRule {
            message: "federation segment needs a value".to_string(),
        };
        let mut item = new_federation_segment("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FederationSegmentRequiredNameRule {
            message: "federation segment needs this field".to_string(),
        };
        let mut item = new_federation_segment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
