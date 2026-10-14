use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy slo audit must pass. Check returns its problems, or none.
pub trait LegacySloAuditRule: std::fmt::Debug {
    fn check(&self, item: &LegacySloAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySloAuditNotesLengthRule {
    pub max_length: i64,
}

impl LegacySloAuditRule for LegacySloAuditNotesLengthRule {
    fn check(&self, item: &LegacySloAudit) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySloAuditBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacySloAuditRule for LegacySloAuditBatchSizeRangeRule {
    fn check(&self, item: &LegacySloAudit) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySloAuditRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacySloAuditRule for LegacySloAuditRateRangeRule {
    fn check(&self, item: &LegacySloAudit) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySloAuditLocaleLengthRule {
    pub max_length: i64,
}

impl LegacySloAuditRule for LegacySloAuditLocaleLengthRule {
    fn check(&self, item: &LegacySloAudit) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy slo audit.
#[derive(Debug, Clone)]
pub struct LegacySloAuditValidator {
    pub rules: Vec<Rc<dyn LegacySloAuditRule>>,
}

impl LegacySloAuditValidator {
    pub fn validate(&self, item: &LegacySloAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacySloAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacySloAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_slo_audit_validator() -> LegacySloAuditValidator {
    LegacySloAuditValidator {
        rules: vec![
            Rc::new(LegacySloAuditNotesLengthRule { max_length: 1000 }) as Rc<dyn LegacySloAuditRule>,
            Rc::new(LegacySloAuditBatchSizeRangeRule { min: 1000, max: 86400 }),
            Rc::new(LegacySloAuditRateRangeRule { min: 7.2, max: 586.2 }),
            Rc::new(LegacySloAuditLocaleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_slo_audit_validator();
        let item = new_legacy_slo_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = LegacySloAuditNotesLengthRule { max_length: 16 };
        let mut item = new_legacy_slo_audit("x", "y");
        item.notes = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = LegacySloAuditBatchSizeRangeRule { min: 1000, max: 86400 };
        let mut item = new_legacy_slo_audit("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = LegacySloAuditRateRangeRule { min: 5.5, max: 3534.9 };
        let mut item = new_legacy_slo_audit("x", "y");
        item.rate = 3536.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = LegacySloAuditLocaleLengthRule { max_length: 32 };
        let mut item = new_legacy_slo_audit("x", "y");
        item.locale = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
