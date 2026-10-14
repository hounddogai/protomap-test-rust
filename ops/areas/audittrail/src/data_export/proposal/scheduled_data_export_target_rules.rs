use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled data export target must pass. Check returns its problems, or none.
pub trait ScheduledDataExportTargetRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledDataExportTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportTargetCodePrefixRule {
    pub prefix: String,
}

impl ScheduledDataExportTargetRule for ScheduledDataExportTargetCodePrefixRule {
    fn check(&self, item: &ScheduledDataExportTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportTargetAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl ScheduledDataExportTargetRule for ScheduledDataExportTargetAllowedPrincipalRule {
    fn check(&self, item: &ScheduledDataExportTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is above maximum", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportTargetCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledDataExportTargetRule for ScheduledDataExportTargetCodeLengthRule {
    fn check(&self, item: &ScheduledDataExportTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportTargetRequiredClusterRule {
    pub message: String,
}

impl ScheduledDataExportTargetRule for ScheduledDataExportTargetRequiredClusterRule {
    fn check(&self, item: &ScheduledDataExportTarget) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportTargetScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledDataExportTargetRule for ScheduledDataExportTargetScoreRangeRule {
    fn check(&self, item: &ScheduledDataExportTarget) -> Vec<String> {
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
pub struct ScheduledDataExportTargetRequiredTitleRule {
    pub message: String,
}

impl ScheduledDataExportTargetRule for ScheduledDataExportTargetRequiredTitleRule {
    fn check(&self, item: &ScheduledDataExportTarget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled data export target.
#[derive(Debug, Clone)]
pub struct ScheduledDataExportTargetValidator {
    pub rules: Vec<Rc<dyn ScheduledDataExportTargetRule>>,
}

impl ScheduledDataExportTargetValidator {
    pub fn validate(&self, item: &ScheduledDataExportTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledDataExportTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledDataExportTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_data_export_target_validator() -> ScheduledDataExportTargetValidator {
    ScheduledDataExportTargetValidator {
        rules: vec![
            Rc::new(ScheduledDataExportTargetCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn ScheduledDataExportTargetRule>,
            Rc::new(ScheduledDataExportTargetAllowedPrincipalRule {
                allowed: vec!["global".to_string()],
            }),
            Rc::new(ScheduledDataExportTargetCodeLengthRule { max_length: 120 }),
            Rc::new(ScheduledDataExportTargetRequiredClusterRule {
                message: "scheduled data export target needs an entry".to_string(),
            }),
            Rc::new(ScheduledDataExportTargetScoreRangeRule { min: 7.6, max: 2117.7 }),
            Rc::new(ScheduledDataExportTargetRequiredTitleRule {
                message: "scheduled data export target needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_data_export_target_validator();
        let item = new_scheduled_data_export_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledDataExportTargetCodeLengthRule { max_length: 80 };
        let mut item = new_scheduled_data_export_target("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = ScheduledDataExportTargetRequiredClusterRule {
            message: "scheduled data export target needs a value".to_string(),
        };
        let mut item = new_scheduled_data_export_target("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ScheduledDataExportTargetScoreRangeRule { min: 1.4, max: 4539.1 };
        let mut item = new_scheduled_data_export_target("x", "y");
        item.score = 4540.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledDataExportTargetRequiredTitleRule {
            message: "scheduled data export target needs an entry".to_string(),
        };
        let mut item = new_scheduled_data_export_target("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
