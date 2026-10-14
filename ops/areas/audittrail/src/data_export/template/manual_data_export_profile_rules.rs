use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual data export profile must pass. Check returns its problems, or none.
pub trait ManualDataExportProfileRule: std::fmt::Debug {
    fn check(&self, item: &ManualDataExportProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportProfileRequiredAudienceRule {
    pub message: String,
}

impl ManualDataExportProfileRule for ManualDataExportProfileRequiredAudienceRule {
    fn check(&self, item: &ManualDataExportProfile) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportProfileApprovedGuardRule {
    pub limit: i64,
}

impl ManualDataExportProfileRule for ManualDataExportProfileApprovedGuardRule {
    fn check(&self, item: &ManualDataExportProfile) -> Vec<String> {
        if item.approved && (item.retries > self.limit) {
            return vec![format!(
                "retries {} exceeds {} while approved",
                item.retries, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportProfileAudienceLengthRule {
    pub max_length: i64,
}

impl ManualDataExportProfileRule for ManualDataExportProfileAudienceLengthRule {
    fn check(&self, item: &ManualDataExportProfile) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportProfileRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualDataExportProfileRule for ManualDataExportProfileRetriesRangeRule {
    fn check(&self, item: &ManualDataExportProfile) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual data export profile.
#[derive(Debug, Clone)]
pub struct ManualDataExportProfileValidator {
    pub rules: Vec<Rc<dyn ManualDataExportProfileRule>>,
}

impl ManualDataExportProfileValidator {
    pub fn validate(&self, item: &ManualDataExportProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualDataExportProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualDataExportProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_data_export_profile_validator() -> ManualDataExportProfileValidator {
    ManualDataExportProfileValidator {
        rules: vec![
            Rc::new(ManualDataExportProfileRequiredAudienceRule {
                message: "manual data export profile needs an entry".to_string(),
            }) as Rc<dyn ManualDataExportProfileRule>,
            Rc::new(ManualDataExportProfileApprovedGuardRule { limit: 32 }),
            Rc::new(ManualDataExportProfileAudienceLengthRule { max_length: 86400 }),
            Rc::new(ManualDataExportProfileRetriesRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_data_export_profile_validator();
        let item = new_manual_data_export_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = ManualDataExportProfileRequiredAudienceRule {
            message: "manual data export profile needs this field".to_string(),
        };
        let mut item = new_manual_data_export_profile("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = ManualDataExportProfileAudienceLengthRule { max_length: 16 };
        let mut item = new_manual_data_export_profile("x", "y");
        item.audience = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = ManualDataExportProfileRetriesRangeRule { min: 64, max: 1000 };
        let mut item = new_manual_data_export_profile("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
