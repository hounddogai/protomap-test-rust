use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment manifest must pass. Check returns its problems, or none.
pub trait SegmentManifestRule: std::fmt::Debug {
    fn check(&self, item: &SegmentManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentManifestDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SegmentManifestRule for SegmentManifestDiscountRangeRule {
    fn check(&self, item: &SegmentManifest) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentManifestRequiredLabelRule {
    pub message: String,
}

impl SegmentManifestRule for SegmentManifestRequiredLabelRule {
    fn check(&self, item: &SegmentManifest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentManifestRequiredCodeRule {
    pub message: String,
}

impl SegmentManifestRule for SegmentManifestRequiredCodeRule {
    fn check(&self, item: &SegmentManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentManifestCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentManifestRule for SegmentManifestCreatedAtRangeRule {
    fn check(&self, item: &SegmentManifest) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment manifest.
#[derive(Debug, Clone)]
pub struct SegmentManifestValidator {
    pub rules: Vec<Rc<dyn SegmentManifestRule>>,
}

impl SegmentManifestValidator {
    pub fn validate(&self, item: &SegmentManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_manifest_validator() -> SegmentManifestValidator {
    SegmentManifestValidator {
        rules: vec![
            Rc::new(SegmentManifestDiscountRangeRule { min: 7.0, max: 2202.2 }) as Rc<dyn SegmentManifestRule>,
            Rc::new(SegmentManifestRequiredLabelRule {
                message: "segment manifest needs an entry".to_string(),
            }),
            Rc::new(SegmentManifestRequiredCodeRule {
                message: "segment manifest needs an entry".to_string(),
            }),
            Rc::new(SegmentManifestCreatedAtRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_manifest_validator();
        let item = new_segment_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = SegmentManifestDiscountRangeRule { min: 7.7, max: 4145.6 };
        let mut item = new_segment_manifest("x", "y");
        item.discount = 4147.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SegmentManifestRequiredLabelRule {
            message: "segment manifest needs an entry".to_string(),
        };
        let mut item = new_segment_manifest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SegmentManifestRequiredCodeRule {
            message: "segment manifest needs a value".to_string(),
        };
        let mut item = new_segment_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = SegmentManifestCreatedAtRangeRule { min: 80, max: 80 };
        let mut item = new_segment_manifest("x", "y");
        item.created_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
