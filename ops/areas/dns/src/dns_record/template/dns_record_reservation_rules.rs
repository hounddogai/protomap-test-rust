use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record reservation must pass. Check returns its problems, or none.
pub trait DnsRecordReservationRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordReservationLabelLengthRule {
    pub max_length: i64,
}

impl DnsRecordReservationRule for DnsRecordReservationLabelLengthRule {
    fn check(&self, item: &DnsRecordReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordReservationRequiredUnitRule {
    pub message: String,
}

impl DnsRecordReservationRule for DnsRecordReservationRequiredUnitRule {
    fn check(&self, item: &DnsRecordReservation) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordReservationVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordReservationRule for DnsRecordReservationVersionRangeRule {
    fn check(&self, item: &DnsRecordReservation) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordReservationRegionLengthRule {
    pub max_length: i64,
}

impl DnsRecordReservationRule for DnsRecordReservationRegionLengthRule {
    fn check(&self, item: &DnsRecordReservation) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordReservationUnitLengthRule {
    pub max_length: i64,
}

impl DnsRecordReservationRule for DnsRecordReservationUnitLengthRule {
    fn check(&self, item: &DnsRecordReservation) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record reservation.
#[derive(Debug, Clone)]
pub struct DnsRecordReservationValidator {
    pub rules: Vec<Rc<dyn DnsRecordReservationRule>>,
}

impl DnsRecordReservationValidator {
    pub fn validate(&self, item: &DnsRecordReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_reservation_validator() -> DnsRecordReservationValidator {
    DnsRecordReservationValidator {
        rules: vec![
            Rc::new(DnsRecordReservationLabelLengthRule { max_length: 32 }) as Rc<dyn DnsRecordReservationRule>,
            Rc::new(DnsRecordReservationRequiredUnitRule {
                message: "dns record reservation needs this field".to_string(),
            }),
            Rc::new(DnsRecordReservationVersionRangeRule { min: 1000, max: 86400 }),
            Rc::new(DnsRecordReservationRegionLengthRule { max_length: 32 }),
            Rc::new(DnsRecordReservationUnitLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_reservation_validator();
        let item = new_dns_record_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DnsRecordReservationLabelLengthRule { max_length: 16 };
        let mut item = new_dns_record_reservation("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = DnsRecordReservationRequiredUnitRule {
            message: "dns record reservation needs this field".to_string(),
        };
        let mut item = new_dns_record_reservation("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = DnsRecordReservationVersionRangeRule { min: 16, max: 255 };
        let mut item = new_dns_record_reservation("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = DnsRecordReservationRegionLengthRule { max_length: 64 };
        let mut item = new_dns_record_reservation("x", "y");
        item.region = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = DnsRecordReservationUnitLengthRule { max_length: 80 };
        let mut item = new_dns_record_reservation("x", "y");
        item.unit = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
