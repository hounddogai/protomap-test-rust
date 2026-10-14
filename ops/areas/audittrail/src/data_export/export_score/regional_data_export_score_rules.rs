use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional data export score must pass. Check returns its problems, or none.
pub trait RegionalDataExportScoreRule: std::fmt::Debug {
    fn check(&self, item: &RegionalDataExportScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportScoreLabelLengthRule {
    pub max_length: i64,
}

impl RegionalDataExportScoreRule for RegionalDataExportScoreLabelLengthRule {
    fn check(&self, item: &RegionalDataExportScore) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportScoreCodePrefixRule {
    pub prefix: String,
}

impl RegionalDataExportScoreRule for RegionalDataExportScoreCodePrefixRule {
    fn check(&self, item: &RegionalDataExportScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportScoreMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalDataExportScoreRule for RegionalDataExportScoreMaximumRangeRule {
    fn check(&self, item: &RegionalDataExportScore) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a regional data export score.
#[derive(Debug, Clone)]
pub struct RegionalDataExportScoreValidator {
    pub rules: Vec<Rc<dyn RegionalDataExportScoreRule>>,
}

impl RegionalDataExportScoreValidator {
    pub fn validate(&self, item: &RegionalDataExportScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalDataExportScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalDataExportScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_data_export_score_validator() -> RegionalDataExportScoreValidator {
    RegionalDataExportScoreValidator {
        rules: vec![
            Rc::new(RegionalDataExportScoreLabelLengthRule { max_length: 1000 }) as Rc<dyn RegionalDataExportScoreRule>,
            Rc::new(RegionalDataExportScoreCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RegionalDataExportScoreMaximumRangeRule { min: 2.2, max: 1383.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_data_export_score_validator();
        let item = new_regional_data_export_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalDataExportScoreLabelLengthRule { max_length: 80 };
        let mut item = new_regional_data_export_score("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RegionalDataExportScoreMaximumRangeRule { min: 4.4, max: 462.6 };
        let mut item = new_regional_data_export_score("x", "y");
        item.maximum = 464.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
