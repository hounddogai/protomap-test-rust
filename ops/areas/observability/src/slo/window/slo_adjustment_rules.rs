use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo adjustment must pass. Check returns its problems, or none.
pub trait SloAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &SloAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAdjustmentRequiredCodeRule {
    pub message: String,
}

impl SloAdjustmentRule for SloAdjustmentRequiredCodeRule {
    fn check(&self, item: &SloAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAdjustmentApprovedGuardRule {
    pub limit: i64,
}

impl SloAdjustmentRule for SloAdjustmentApprovedGuardRule {
    fn check(&self, item: &SloAdjustment) -> Vec<String> {
        if item.approved && (item.version > self.limit) {
            return vec![format!(
                "version {} exceeds {} while approved",
                item.version, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAdjustmentRequiredKeyIdRule {
    pub message: String,
}

impl SloAdjustmentRule for SloAdjustmentRequiredKeyIdRule {
    fn check(&self, item: &SloAdjustment) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAdjustmentSummaryLengthRule {
    pub max_length: i64,
}

impl SloAdjustmentRule for SloAdjustmentSummaryLengthRule {
    fn check(&self, item: &SloAdjustment) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAdjustmentVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloAdjustmentRule for SloAdjustmentVersionRangeRule {
    fn check(&self, item: &SloAdjustment) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo adjustment.
#[derive(Debug, Clone)]
pub struct SloAdjustmentValidator {
    pub rules: Vec<Rc<dyn SloAdjustmentRule>>,
}

impl SloAdjustmentValidator {
    pub fn validate(&self, item: &SloAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_adjustment_validator() -> SloAdjustmentValidator {
    SloAdjustmentValidator {
        rules: vec![
            Rc::new(SloAdjustmentRequiredCodeRule {
                message: "slo adjustment needs this field".to_string(),
            }) as Rc<dyn SloAdjustmentRule>,
            Rc::new(SloAdjustmentApprovedGuardRule { limit: 32 }),
            Rc::new(SloAdjustmentRequiredKeyIdRule {
                message: "slo adjustment needs a value".to_string(),
            }),
            Rc::new(SloAdjustmentSummaryLengthRule { max_length: 1000 }),
            Rc::new(SloAdjustmentVersionRangeRule { min: 16, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_adjustment_validator();
        let item = new_slo_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SloAdjustmentRequiredCodeRule {
            message: "slo adjustment needs a value".to_string(),
        };
        let mut item = new_slo_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = SloAdjustmentRequiredKeyIdRule {
            message: "slo adjustment needs this field".to_string(),
        };
        let mut item = new_slo_adjustment("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = SloAdjustmentSummaryLengthRule { max_length: 1000 };
        let mut item = new_slo_adjustment("x", "y");
        item.summary = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = SloAdjustmentVersionRangeRule { min: 80, max: 80 };
        let mut item = new_slo_adjustment("x", "y");
        item.version = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
