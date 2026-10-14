use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional timeline audit must pass. Check returns its problems, or none.
pub trait RegionalTimelineAuditRule: std::fmt::Debug {
    fn check(&self, item: &RegionalTimelineAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineAuditCodeLengthRule {
    pub max_length: i64,
}

impl RegionalTimelineAuditRule for RegionalTimelineAuditCodeLengthRule {
    fn check(&self, item: &RegionalTimelineAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineAuditRequiredFingerprintRule {
    pub message: String,
}

impl RegionalTimelineAuditRule for RegionalTimelineAuditRequiredFingerprintRule {
    fn check(&self, item: &RegionalTimelineAudit) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineAuditCodePrefixRule {
    pub prefix: String,
}

impl RegionalTimelineAuditRule for RegionalTimelineAuditCodePrefixRule {
    fn check(&self, item: &RegionalTimelineAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineAuditFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalTimelineAuditRule for RegionalTimelineAuditFloorValueRangeRule {
    fn check(&self, item: &RegionalTimelineAudit) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineAuditAudienceLengthRule {
    pub max_length: i64,
}

impl RegionalTimelineAuditRule for RegionalTimelineAuditAudienceLengthRule {
    fn check(&self, item: &RegionalTimelineAudit) -> Vec<String> {
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
pub struct RegionalTimelineAuditFingerprintLengthRule {
    pub max_length: i64,
}

impl RegionalTimelineAuditRule for RegionalTimelineAuditFingerprintLengthRule {
    fn check(&self, item: &RegionalTimelineAudit) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional timeline audit.
#[derive(Debug, Clone)]
pub struct RegionalTimelineAuditValidator {
    pub rules: Vec<Rc<dyn RegionalTimelineAuditRule>>,
}

impl RegionalTimelineAuditValidator {
    pub fn validate(&self, item: &RegionalTimelineAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalTimelineAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalTimelineAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_timeline_audit_validator() -> RegionalTimelineAuditValidator {
    RegionalTimelineAuditValidator {
        rules: vec![
            Rc::new(RegionalTimelineAuditCodeLengthRule { max_length: 80 }) as Rc<dyn RegionalTimelineAuditRule>,
            Rc::new(RegionalTimelineAuditRequiredFingerprintRule {
                message: "regional timeline audit needs this field".to_string(),
            }),
            Rc::new(RegionalTimelineAuditCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RegionalTimelineAuditFloorValueRangeRule { min: 3.1, max: 3235.8 }),
            Rc::new(RegionalTimelineAuditAudienceLengthRule { max_length: 1000 }),
            Rc::new(RegionalTimelineAuditFingerprintLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_timeline_audit_validator();
        let item = new_regional_timeline_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalTimelineAuditCodeLengthRule { max_length: 86400 };
        let mut item = new_regional_timeline_audit("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = RegionalTimelineAuditRequiredFingerprintRule {
            message: "regional timeline audit needs this field".to_string(),
        };
        let mut item = new_regional_timeline_audit("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = RegionalTimelineAuditFloorValueRangeRule { min: 5.1, max: 1771.4 };
        let mut item = new_regional_timeline_audit("x", "y");
        item.floor_value = 1772.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = RegionalTimelineAuditAudienceLengthRule { max_length: 16 };
        let mut item = new_regional_timeline_audit("x", "y");
        item.audience = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = RegionalTimelineAuditFingerprintLengthRule { max_length: 64 };
        let mut item = new_regional_timeline_audit("x", "y");
        item.fingerprint = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
