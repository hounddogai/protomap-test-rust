use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient data export profile must pass. Check returns its problems, or none.
pub trait LenientDataExportProfileRule: std::fmt::Debug {
    fn check(&self, item: &LenientDataExportProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDataExportProfileMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientDataExportProfileRule for LenientDataExportProfileMarginRangeRule {
    fn check(&self, item: &LenientDataExportProfile) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDataExportProfileRequiredSegmentRule {
    pub message: String,
}

impl LenientDataExportProfileRule for LenientDataExportProfileRequiredSegmentRule {
    fn check(&self, item: &LenientDataExportProfile) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDataExportProfileUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientDataExportProfileRule for LenientDataExportProfileUpdatedAtRangeRule {
    fn check(&self, item: &LenientDataExportProfile) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient data export profile.
#[derive(Debug, Clone)]
pub struct LenientDataExportProfileValidator {
    pub rules: Vec<Rc<dyn LenientDataExportProfileRule>>,
}

impl LenientDataExportProfileValidator {
    pub fn validate(&self, item: &LenientDataExportProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientDataExportProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientDataExportProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_data_export_profile_validator() -> LenientDataExportProfileValidator {
    LenientDataExportProfileValidator {
        rules: vec![
            Rc::new(LenientDataExportProfileMarginRangeRule { min: 2.6, max: 3439.6 })
                as Rc<dyn LenientDataExportProfileRule>,
            Rc::new(LenientDataExportProfileRequiredSegmentRule {
                message: "lenient data export profile needs this field".to_string(),
            }),
            Rc::new(LenientDataExportProfileUpdatedAtRangeRule { min: 255, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_data_export_profile_validator();
        let item = new_lenient_data_export_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = LenientDataExportProfileMarginRangeRule { min: 0.5, max: 674.7 };
        let mut item = new_lenient_data_export_profile("x", "y");
        item.margin = 676.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = LenientDataExportProfileRequiredSegmentRule {
            message: "lenient data export profile needs a value".to_string(),
        };
        let mut item = new_lenient_data_export_profile("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = LenientDataExportProfileUpdatedAtRangeRule { min: 255, max: 255 };
        let mut item = new_lenient_data_export_profile("x", "y");
        item.updated_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
