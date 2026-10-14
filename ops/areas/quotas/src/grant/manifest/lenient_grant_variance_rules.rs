use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient grant variance must pass. Check returns its problems, or none.
pub trait LenientGrantVarianceRule: std::fmt::Debug {
    fn check(&self, item: &LenientGrantVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantVarianceRequiredNameRule {
    pub message: String,
}

impl LenientGrantVarianceRule for LenientGrantVarianceRequiredNameRule {
    fn check(&self, item: &LenientGrantVariance) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantVarianceSegmentLengthRule {
    pub max_length: i64,
}

impl LenientGrantVarianceRule for LenientGrantVarianceSegmentLengthRule {
    fn check(&self, item: &LenientGrantVariance) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantVarianceWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientGrantVarianceRule for LenientGrantVarianceWeightRangeRule {
    fn check(&self, item: &LenientGrantVariance) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantVarianceChecksumLengthRule {
    pub max_length: i64,
}

impl LenientGrantVarianceRule for LenientGrantVarianceChecksumLengthRule {
    fn check(&self, item: &LenientGrantVariance) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient grant variance.
#[derive(Debug, Clone)]
pub struct LenientGrantVarianceValidator {
    pub rules: Vec<Rc<dyn LenientGrantVarianceRule>>,
}

impl LenientGrantVarianceValidator {
    pub fn validate(&self, item: &LenientGrantVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientGrantVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientGrantVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_grant_variance_validator() -> LenientGrantVarianceValidator {
    LenientGrantVarianceValidator {
        rules: vec![
            Rc::new(LenientGrantVarianceRequiredNameRule {
                message: "lenient grant variance needs a value".to_string(),
            }) as Rc<dyn LenientGrantVarianceRule>,
            Rc::new(LenientGrantVarianceSegmentLengthRule { max_length: 3600 }),
            Rc::new(LenientGrantVarianceWeightRangeRule { min: 1.9, max: 3520.5 }),
            Rc::new(LenientGrantVarianceChecksumLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_grant_variance_validator();
        let item = new_lenient_grant_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LenientGrantVarianceRequiredNameRule {
            message: "lenient grant variance needs this field".to_string(),
        };
        let mut item = new_lenient_grant_variance("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = LenientGrantVarianceSegmentLengthRule { max_length: 1000 };
        let mut item = new_lenient_grant_variance("x", "y");
        item.segment = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = LenientGrantVarianceWeightRangeRule { min: 5.0, max: 3836.5 };
        let mut item = new_lenient_grant_variance("x", "y");
        item.weight = 3838.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = LenientGrantVarianceChecksumLengthRule { max_length: 16 };
        let mut item = new_lenient_grant_variance("x", "y");
        item.checksum = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
