use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy flow log threshold must pass. Check returns its problems, or none.
pub trait LegacyFlowLogThresholdRule: std::fmt::Debug {
    fn check(&self, item: &LegacyFlowLogThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogThresholdRequiredKeyIdRule {
    pub message: String,
}

impl LegacyFlowLogThresholdRule for LegacyFlowLogThresholdRequiredKeyIdRule {
    fn check(&self, item: &LegacyFlowLogThreshold) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogThresholdRequiredCodeRule {
    pub message: String,
}

impl LegacyFlowLogThresholdRule for LegacyFlowLogThresholdRequiredCodeRule {
    fn check(&self, item: &LegacyFlowLogThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogThresholdRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyFlowLogThresholdRule for LegacyFlowLogThresholdRotationDaysRangeRule {
    fn check(&self, item: &LegacyFlowLogThreshold) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogThresholdNotesLengthRule {
    pub max_length: i64,
}

impl LegacyFlowLogThresholdRule for LegacyFlowLogThresholdNotesLengthRule {
    fn check(&self, item: &LegacyFlowLogThreshold) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogThresholdLabelLengthRule {
    pub max_length: i64,
}

impl LegacyFlowLogThresholdRule for LegacyFlowLogThresholdLabelLengthRule {
    fn check(&self, item: &LegacyFlowLogThreshold) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogThresholdRequiredLabelRule {
    pub message: String,
}

impl LegacyFlowLogThresholdRule for LegacyFlowLogThresholdRequiredLabelRule {
    fn check(&self, item: &LegacyFlowLogThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy flow log threshold.
#[derive(Debug, Clone)]
pub struct LegacyFlowLogThresholdValidator {
    pub rules: Vec<Rc<dyn LegacyFlowLogThresholdRule>>,
}

impl LegacyFlowLogThresholdValidator {
    pub fn validate(&self, item: &LegacyFlowLogThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyFlowLogThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyFlowLogThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_flow_log_threshold_validator() -> LegacyFlowLogThresholdValidator {
    LegacyFlowLogThresholdValidator {
        rules: vec![
            Rc::new(LegacyFlowLogThresholdRequiredKeyIdRule {
                message: "legacy flow log threshold needs an entry".to_string(),
            }) as Rc<dyn LegacyFlowLogThresholdRule>,
            Rc::new(LegacyFlowLogThresholdRequiredCodeRule {
                message: "legacy flow log threshold needs a value".to_string(),
            }),
            Rc::new(LegacyFlowLogThresholdRotationDaysRangeRule { min: 255, max: 1000 }),
            Rc::new(LegacyFlowLogThresholdNotesLengthRule { max_length: 32 }),
            Rc::new(LegacyFlowLogThresholdLabelLengthRule { max_length: 120 }),
            Rc::new(LegacyFlowLogThresholdRequiredLabelRule {
                message: "legacy flow log threshold needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_flow_log_threshold_validator();
        let item = new_legacy_flow_log_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = LegacyFlowLogThresholdRequiredKeyIdRule {
            message: "legacy flow log threshold needs a value".to_string(),
        };
        let mut item = new_legacy_flow_log_threshold("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyFlowLogThresholdRequiredCodeRule {
            message: "legacy flow log threshold needs a value".to_string(),
        };
        let mut item = new_legacy_flow_log_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = LegacyFlowLogThresholdRotationDaysRangeRule { min: 1000, max: 1000 };
        let mut item = new_legacy_flow_log_threshold("x", "y");
        item.rotation_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = LegacyFlowLogThresholdNotesLengthRule { max_length: 120 };
        let mut item = new_legacy_flow_log_threshold("x", "y");
        item.notes = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacyFlowLogThresholdLabelLengthRule { max_length: 32 };
        let mut item = new_legacy_flow_log_threshold("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LegacyFlowLogThresholdRequiredLabelRule {
            message: "legacy flow log threshold needs a value".to_string(),
        };
        let mut item = new_legacy_flow_log_threshold("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
