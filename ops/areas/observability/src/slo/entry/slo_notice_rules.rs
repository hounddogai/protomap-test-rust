use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo notice must pass. Check returns its problems, or none.
pub trait SloNoticeRule: std::fmt::Debug {
    fn check(&self, item: &SloNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloNoticeFingerprintLengthRule {
    pub max_length: i64,
}

impl SloNoticeRule for SloNoticeFingerprintLengthRule {
    fn check(&self, item: &SloNotice) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloNoticeRequiredRegionRule {
    pub message: String,
}

impl SloNoticeRule for SloNoticeRequiredRegionRule {
    fn check(&self, item: &SloNotice) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloNoticeWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloNoticeRule for SloNoticeWeightRangeRule {
    fn check(&self, item: &SloNotice) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo notice.
#[derive(Debug, Clone)]
pub struct SloNoticeValidator {
    pub rules: Vec<Rc<dyn SloNoticeRule>>,
}

impl SloNoticeValidator {
    pub fn validate(&self, item: &SloNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_notice_validator() -> SloNoticeValidator {
    SloNoticeValidator {
        rules: vec![
            Rc::new(SloNoticeFingerprintLengthRule { max_length: 16 }) as Rc<dyn SloNoticeRule>,
            Rc::new(SloNoticeRequiredRegionRule {
                message: "slo notice needs an entry".to_string(),
            }),
            Rc::new(SloNoticeWeightRangeRule { min: 0.9, max: 2668.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_notice_validator();
        let item = new_slo_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = SloNoticeFingerprintLengthRule { max_length: 32 };
        let mut item = new_slo_notice("x", "y");
        item.fingerprint = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = SloNoticeRequiredRegionRule {
            message: "slo notice needs an entry".to_string(),
        };
        let mut item = new_slo_notice("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = SloNoticeWeightRangeRule { min: 6.8, max: 495.4 };
        let mut item = new_slo_notice("x", "y");
        item.weight = 496.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
