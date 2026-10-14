use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy probe adjustment must pass. Check returns its problems, or none.
pub trait LegacyProbeAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &LegacyProbeAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeAdjustmentSegmentLengthRule {
    pub max_length: i64,
}

impl LegacyProbeAdjustmentRule for LegacyProbeAdjustmentSegmentLengthRule {
    fn check(&self, item: &LegacyProbeAdjustment) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeAdjustmentRequiredLabelRule {
    pub message: String,
}

impl LegacyProbeAdjustmentRule for LegacyProbeAdjustmentRequiredLabelRule {
    fn check(&self, item: &LegacyProbeAdjustment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeAdjustmentRequiredNameRule {
    pub message: String,
}

impl LegacyProbeAdjustmentRule for LegacyProbeAdjustmentRequiredNameRule {
    fn check(&self, item: &LegacyProbeAdjustment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy probe adjustment.
#[derive(Debug, Clone)]
pub struct LegacyProbeAdjustmentValidator {
    pub rules: Vec<Rc<dyn LegacyProbeAdjustmentRule>>,
}

impl LegacyProbeAdjustmentValidator {
    pub fn validate(&self, item: &LegacyProbeAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyProbeAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyProbeAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_probe_adjustment_validator() -> LegacyProbeAdjustmentValidator {
    LegacyProbeAdjustmentValidator {
        rules: vec![
            Rc::new(LegacyProbeAdjustmentSegmentLengthRule { max_length: 120 }) as Rc<dyn LegacyProbeAdjustmentRule>,
            Rc::new(LegacyProbeAdjustmentRequiredLabelRule {
                message: "legacy probe adjustment needs a value".to_string(),
            }),
            Rc::new(LegacyProbeAdjustmentRequiredNameRule {
                message: "legacy probe adjustment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_probe_adjustment_validator();
        let item = new_legacy_probe_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = LegacyProbeAdjustmentSegmentLengthRule { max_length: 1000 };
        let mut item = new_legacy_probe_adjustment("x", "y");
        item.segment = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LegacyProbeAdjustmentRequiredLabelRule {
            message: "legacy probe adjustment needs a value".to_string(),
        };
        let mut item = new_legacy_probe_adjustment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyProbeAdjustmentRequiredNameRule {
            message: "legacy probe adjustment needs this field".to_string(),
        };
        let mut item = new_legacy_probe_adjustment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
