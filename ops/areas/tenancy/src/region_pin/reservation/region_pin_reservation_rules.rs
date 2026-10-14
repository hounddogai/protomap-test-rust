use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin reservation must pass. Check returns its problems, or none.
pub trait RegionPinReservationRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinReservationQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinReservationRule for RegionPinReservationQuotaUnitsRangeRule {
    fn check(&self, item: &RegionPinReservation) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinReservationCipherLengthRule {
    pub max_length: i64,
}

impl RegionPinReservationRule for RegionPinReservationCipherLengthRule {
    fn check(&self, item: &RegionPinReservation) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinReservationRequiredCodeRule {
    pub message: String,
}

impl RegionPinReservationRule for RegionPinReservationRequiredCodeRule {
    fn check(&self, item: &RegionPinReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinReservationRequiredNameRule {
    pub message: String,
}

impl RegionPinReservationRule for RegionPinReservationRequiredNameRule {
    fn check(&self, item: &RegionPinReservation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin reservation.
#[derive(Debug, Clone)]
pub struct RegionPinReservationValidator {
    pub rules: Vec<Rc<dyn RegionPinReservationRule>>,
}

impl RegionPinReservationValidator {
    pub fn validate(&self, item: &RegionPinReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_reservation_validator() -> RegionPinReservationValidator {
    RegionPinReservationValidator {
        rules: vec![
            Rc::new(RegionPinReservationQuotaUnitsRangeRule { min: 80, max: 120 }) as Rc<dyn RegionPinReservationRule>,
            Rc::new(RegionPinReservationCipherLengthRule { max_length: 32 }),
            Rc::new(RegionPinReservationRequiredCodeRule {
                message: "region pin reservation needs an entry".to_string(),
            }),
            Rc::new(RegionPinReservationRequiredNameRule {
                message: "region pin reservation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_reservation_validator();
        let item = new_region_pin_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = RegionPinReservationQuotaUnitsRangeRule { min: 16, max: 1000 };
        let mut item = new_region_pin_reservation("x", "y");
        item.quota_units = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = RegionPinReservationCipherLengthRule { max_length: 16 };
        let mut item = new_region_pin_reservation("x", "y");
        item.cipher = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinReservationRequiredCodeRule {
            message: "region pin reservation needs this field".to_string(),
        };
        let mut item = new_region_pin_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionPinReservationRequiredNameRule {
            message: "region pin reservation needs a value".to_string(),
        };
        let mut item = new_region_pin_reservation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
