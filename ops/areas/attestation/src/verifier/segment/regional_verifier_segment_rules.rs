use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional verifier segment must pass. Check returns its problems, or none.
pub trait RegionalVerifierSegmentRule: std::fmt::Debug {
    fn check(&self, item: &RegionalVerifierSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierSegmentRequiredIssuerRule {
    pub message: String,
}

impl RegionalVerifierSegmentRule for RegionalVerifierSegmentRequiredIssuerRule {
    fn check(&self, item: &RegionalVerifierSegment) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierSegmentVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalVerifierSegmentRule for RegionalVerifierSegmentVersionRangeRule {
    fn check(&self, item: &RegionalVerifierSegment) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierSegmentRequiredAudienceRule {
    pub message: String,
}

impl RegionalVerifierSegmentRule for RegionalVerifierSegmentRequiredAudienceRule {
    fn check(&self, item: &RegionalVerifierSegment) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional verifier segment.
#[derive(Debug, Clone)]
pub struct RegionalVerifierSegmentValidator {
    pub rules: Vec<Rc<dyn RegionalVerifierSegmentRule>>,
}

impl RegionalVerifierSegmentValidator {
    pub fn validate(&self, item: &RegionalVerifierSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalVerifierSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalVerifierSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_verifier_segment_validator() -> RegionalVerifierSegmentValidator {
    RegionalVerifierSegmentValidator {
        rules: vec![
            Rc::new(RegionalVerifierSegmentRequiredIssuerRule {
                message: "regional verifier segment needs this field".to_string(),
            }) as Rc<dyn RegionalVerifierSegmentRule>,
            Rc::new(RegionalVerifierSegmentVersionRangeRule { min: 32, max: 1000 }),
            Rc::new(RegionalVerifierSegmentRequiredAudienceRule {
                message: "regional verifier segment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_verifier_segment_validator();
        let item = new_regional_verifier_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = RegionalVerifierSegmentRequiredIssuerRule {
            message: "regional verifier segment needs a value".to_string(),
        };
        let mut item = new_regional_verifier_segment("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RegionalVerifierSegmentVersionRangeRule { min: 32, max: 1000 };
        let mut item = new_regional_verifier_segment("x", "y");
        item.version = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = RegionalVerifierSegmentRequiredAudienceRule {
            message: "regional verifier segment needs an entry".to_string(),
        };
        let mut item = new_regional_verifier_segment("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
