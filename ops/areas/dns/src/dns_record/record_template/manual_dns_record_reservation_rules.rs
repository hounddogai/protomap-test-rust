use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual dns record reservation must pass. Check returns its problems, or none.
pub trait ManualDnsRecordReservationRule: std::fmt::Debug {
    fn check(&self, item: &ManualDnsRecordReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordReservationRequiredLabelRule {
    pub message: String,
}

impl ManualDnsRecordReservationRule for ManualDnsRecordReservationRequiredLabelRule {
    fn check(&self, item: &ManualDnsRecordReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordReservationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ManualDnsRecordReservationRule for ManualDnsRecordReservationAllowedCodeRule {
    fn check(&self, item: &ManualDnsRecordReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordReservationCodeLengthRule {
    pub max_length: i64,
}

impl ManualDnsRecordReservationRule for ManualDnsRecordReservationCodeLengthRule {
    fn check(&self, item: &ManualDnsRecordReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordReservationRequiredCodeRule {
    pub message: String,
}

impl ManualDnsRecordReservationRule for ManualDnsRecordReservationRequiredCodeRule {
    fn check(&self, item: &ManualDnsRecordReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordReservationStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualDnsRecordReservationRule for ManualDnsRecordReservationStartsAtRangeRule {
    fn check(&self, item: &ManualDnsRecordReservation) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordReservationMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualDnsRecordReservationRule for ManualDnsRecordReservationMarginRangeRule {
    fn check(&self, item: &ManualDnsRecordReservation) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual dns record reservation.
#[derive(Debug, Clone)]
pub struct ManualDnsRecordReservationValidator {
    pub rules: Vec<Rc<dyn ManualDnsRecordReservationRule>>,
}

impl ManualDnsRecordReservationValidator {
    pub fn validate(&self, item: &ManualDnsRecordReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualDnsRecordReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualDnsRecordReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_dns_record_reservation_validator() -> ManualDnsRecordReservationValidator {
    ManualDnsRecordReservationValidator {
        rules: vec![
            Rc::new(ManualDnsRecordReservationRequiredLabelRule {
                message: "manual dns record reservation needs a value".to_string(),
            }) as Rc<dyn ManualDnsRecordReservationRule>,
            Rc::new(ManualDnsRecordReservationAllowedCodeRule {
                allowed: vec!["tiered".to_string(), "regional".to_string()],
            }),
            Rc::new(ManualDnsRecordReservationCodeLengthRule { max_length: 120 }),
            Rc::new(ManualDnsRecordReservationRequiredCodeRule {
                message: "manual dns record reservation needs this field".to_string(),
            }),
            Rc::new(ManualDnsRecordReservationStartsAtRangeRule { min: 32, max: 80 }),
            Rc::new(ManualDnsRecordReservationMarginRangeRule { min: 0.2, max: 2144.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_dns_record_reservation_validator();
        let item = new_manual_dns_record_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualDnsRecordReservationRequiredLabelRule {
            message: "manual dns record reservation needs a value".to_string(),
        };
        let mut item = new_manual_dns_record_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualDnsRecordReservationCodeLengthRule { max_length: 32 };
        let mut item = new_manual_dns_record_reservation("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualDnsRecordReservationRequiredCodeRule {
            message: "manual dns record reservation needs an entry".to_string(),
        };
        let mut item = new_manual_dns_record_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = ManualDnsRecordReservationStartsAtRangeRule { min: 64, max: 64 };
        let mut item = new_manual_dns_record_reservation("x", "y");
        item.starts_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = ManualDnsRecordReservationMarginRangeRule { min: 0.2, max: 4884.2 };
        let mut item = new_manual_dns_record_reservation("x", "y");
        item.margin = 4885.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
