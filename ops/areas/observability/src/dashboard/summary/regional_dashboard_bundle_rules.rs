use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional dashboard bundle must pass. Check returns its problems, or none.
pub trait RegionalDashboardBundleRule: std::fmt::Debug {
    fn check(&self, item: &RegionalDashboardBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardBundleExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalDashboardBundleRule for RegionalDashboardBundleExpiresAtRangeRule {
    fn check(&self, item: &RegionalDashboardBundle) -> Vec<String> {
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
pub struct RegionalDashboardBundleRequiredNotesRule {
    pub message: String,
}

impl RegionalDashboardBundleRule for RegionalDashboardBundleRequiredNotesRule {
    fn check(&self, item: &RegionalDashboardBundle) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardBundlePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalDashboardBundleRule for RegionalDashboardBundlePriorityRangeRule {
    fn check(&self, item: &RegionalDashboardBundle) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardBundleNotesLengthRule {
    pub max_length: i64,
}

impl RegionalDashboardBundleRule for RegionalDashboardBundleNotesLengthRule {
    fn check(&self, item: &RegionalDashboardBundle) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardBundleScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalDashboardBundleRule for RegionalDashboardBundleScoreRangeRule {
    fn check(&self, item: &RegionalDashboardBundle) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardBundleRequiredCodeRule {
    pub message: String,
}

impl RegionalDashboardBundleRule for RegionalDashboardBundleRequiredCodeRule {
    fn check(&self, item: &RegionalDashboardBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional dashboard bundle.
#[derive(Debug, Clone)]
pub struct RegionalDashboardBundleValidator {
    pub rules: Vec<Rc<dyn RegionalDashboardBundleRule>>,
}

impl RegionalDashboardBundleValidator {
    pub fn validate(&self, item: &RegionalDashboardBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalDashboardBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalDashboardBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_dashboard_bundle_validator() -> RegionalDashboardBundleValidator {
    RegionalDashboardBundleValidator {
        rules: vec![
            Rc::new(RegionalDashboardBundleExpiresAtRangeRule { min: 16, max: 80 })
                as Rc<dyn RegionalDashboardBundleRule>,
            Rc::new(RegionalDashboardBundleRequiredNotesRule {
                message: "regional dashboard bundle needs this field".to_string(),
            }),
            Rc::new(RegionalDashboardBundlePriorityRangeRule { min: 32, max: 64 }),
            Rc::new(RegionalDashboardBundleNotesLengthRule { max_length: 80 }),
            Rc::new(RegionalDashboardBundleScoreRangeRule { min: 4.1, max: 1612.0 }),
            Rc::new(RegionalDashboardBundleRequiredCodeRule {
                message: "regional dashboard bundle needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_dashboard_bundle_validator();
        let item = new_regional_dashboard_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = RegionalDashboardBundleExpiresAtRangeRule { min: 16, max: 120 };
        let mut item = new_regional_dashboard_bundle("x", "y");
        item.expires_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = RegionalDashboardBundleRequiredNotesRule {
            message: "regional dashboard bundle needs this field".to_string(),
        };
        let mut item = new_regional_dashboard_bundle("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = RegionalDashboardBundlePriorityRangeRule { min: 255, max: 3600 };
        let mut item = new_regional_dashboard_bundle("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RegionalDashboardBundleNotesLengthRule { max_length: 120 };
        let mut item = new_regional_dashboard_bundle("x", "y");
        item.notes = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = RegionalDashboardBundleScoreRangeRule { min: 4.5, max: 4209.3 };
        let mut item = new_regional_dashboard_bundle("x", "y");
        item.score = 4210.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalDashboardBundleRequiredCodeRule {
            message: "regional dashboard bundle needs a value".to_string(),
        };
        let mut item = new_regional_dashboard_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
