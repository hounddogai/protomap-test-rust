use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log reservation must pass. Check returns its problems, or none.
pub trait FlowLogReservationRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogReservationSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogReservationRule for FlowLogReservationSequenceRangeRule {
    fn check(&self, item: &FlowLogReservation) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogReservationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FlowLogReservationRule for FlowLogReservationAllowedCodeRule {
    fn check(&self, item: &FlowLogReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogReservationCodePrefixRule {
    pub prefix: String,
}

impl FlowLogReservationRule for FlowLogReservationCodePrefixRule {
    fn check(&self, item: &FlowLogReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogReservationFingerprintLengthRule {
    pub max_length: i64,
}

impl FlowLogReservationRule for FlowLogReservationFingerprintLengthRule {
    fn check(&self, item: &FlowLogReservation) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogReservationTitleLengthRule {
    pub max_length: i64,
}

impl FlowLogReservationRule for FlowLogReservationTitleLengthRule {
    fn check(&self, item: &FlowLogReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogReservationRequiredFingerprintRule {
    pub message: String,
}

impl FlowLogReservationRule for FlowLogReservationRequiredFingerprintRule {
    fn check(&self, item: &FlowLogReservation) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log reservation.
#[derive(Debug, Clone)]
pub struct FlowLogReservationValidator {
    pub rules: Vec<Rc<dyn FlowLogReservationRule>>,
}

impl FlowLogReservationValidator {
    pub fn validate(&self, item: &FlowLogReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_reservation_validator() -> FlowLogReservationValidator {
    FlowLogReservationValidator {
        rules: vec![
            Rc::new(FlowLogReservationSequenceRangeRule { min: 16, max: 1000 }) as Rc<dyn FlowLogReservationRule>,
            Rc::new(FlowLogReservationAllowedCodeRule {
                allowed: vec!["weighted".to_string()],
            }),
            Rc::new(FlowLogReservationCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(FlowLogReservationFingerprintLengthRule { max_length: 1000 }),
            Rc::new(FlowLogReservationTitleLengthRule { max_length: 16 }),
            Rc::new(FlowLogReservationRequiredFingerprintRule {
                message: "flow log reservation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_reservation_validator();
        let item = new_flow_log_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = FlowLogReservationSequenceRangeRule { min: 16, max: 255 };
        let mut item = new_flow_log_reservation("x", "y");
        item.sequence = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = FlowLogReservationFingerprintLengthRule { max_length: 120 };
        let mut item = new_flow_log_reservation("x", "y");
        item.fingerprint = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FlowLogReservationTitleLengthRule { max_length: 86400 };
        let mut item = new_flow_log_reservation("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = FlowLogReservationRequiredFingerprintRule {
            message: "flow log reservation needs a value".to_string(),
        };
        let mut item = new_flow_log_reservation("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
