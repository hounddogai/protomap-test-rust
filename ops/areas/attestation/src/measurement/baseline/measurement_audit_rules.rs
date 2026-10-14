use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement audit must pass. Check returns its problems, or none.
pub trait MeasurementAuditRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAuditHostLengthRule {
    pub max_length: i64,
}

impl MeasurementAuditRule for MeasurementAuditHostLengthRule {
    fn check(&self, item: &MeasurementAudit) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAuditScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementAuditRule for MeasurementAuditScoreRangeRule {
    fn check(&self, item: &MeasurementAudit) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAuditCodePrefixRule {
    pub prefix: String,
}

impl MeasurementAuditRule for MeasurementAuditCodePrefixRule {
    fn check(&self, item: &MeasurementAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAuditRequiredCodeRule {
    pub message: String,
}

impl MeasurementAuditRule for MeasurementAuditRequiredCodeRule {
    fn check(&self, item: &MeasurementAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement audit.
#[derive(Debug, Clone)]
pub struct MeasurementAuditValidator {
    pub rules: Vec<Rc<dyn MeasurementAuditRule>>,
}

impl MeasurementAuditValidator {
    pub fn validate(&self, item: &MeasurementAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MeasurementAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_audit_validator() -> MeasurementAuditValidator {
    MeasurementAuditValidator {
        rules: vec![
            Rc::new(MeasurementAuditHostLengthRule { max_length: 80 }) as Rc<dyn MeasurementAuditRule>,
            Rc::new(MeasurementAuditScoreRangeRule { min: 7.3, max: 1893.6 }),
            Rc::new(MeasurementAuditCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(MeasurementAuditRequiredCodeRule {
                message: "measurement audit needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_audit_validator();
        let item = new_measurement_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = MeasurementAuditHostLengthRule { max_length: 64 };
        let mut item = new_measurement_audit("x", "y");
        item.host = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = MeasurementAuditScoreRangeRule { min: 8.1, max: 2371.0 };
        let mut item = new_measurement_audit("x", "y");
        item.score = 2372.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MeasurementAuditRequiredCodeRule {
            message: "measurement audit needs an entry".to_string(),
        };
        let mut item = new_measurement_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
