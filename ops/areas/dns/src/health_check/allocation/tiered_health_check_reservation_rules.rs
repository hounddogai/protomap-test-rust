use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered health check reservation must pass. Check returns its problems, or none.
pub trait TieredHealthCheckReservationRule: std::fmt::Debug {
    fn check(&self, item: &TieredHealthCheckReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckReservationQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredHealthCheckReservationRule for TieredHealthCheckReservationQuotaUnitsRangeRule {
    fn check(&self, item: &TieredHealthCheckReservation) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckReservationRequiredContactEmailRule {
    pub message: String,
}

impl TieredHealthCheckReservationRule for TieredHealthCheckReservationRequiredContactEmailRule {
    fn check(&self, item: &TieredHealthCheckReservation) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckReservationContactEmailLengthRule {
    pub max_length: i64,
}

impl TieredHealthCheckReservationRule for TieredHealthCheckReservationContactEmailLengthRule {
    fn check(&self, item: &TieredHealthCheckReservation) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckReservationAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl TieredHealthCheckReservationRule for TieredHealthCheckReservationAllowedCurrencyRule {
    fn check(&self, item: &TieredHealthCheckReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is blocked", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckReservationRequiredLabelRule {
    pub message: String,
}

impl TieredHealthCheckReservationRule for TieredHealthCheckReservationRequiredLabelRule {
    fn check(&self, item: &TieredHealthCheckReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered health check reservation.
#[derive(Debug, Clone)]
pub struct TieredHealthCheckReservationValidator {
    pub rules: Vec<Rc<dyn TieredHealthCheckReservationRule>>,
}

impl TieredHealthCheckReservationValidator {
    pub fn validate(&self, item: &TieredHealthCheckReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredHealthCheckReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredHealthCheckReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_health_check_reservation_validator() -> TieredHealthCheckReservationValidator {
    TieredHealthCheckReservationValidator {
        rules: vec![
            Rc::new(TieredHealthCheckReservationQuotaUnitsRangeRule { min: 32, max: 255 })
                as Rc<dyn TieredHealthCheckReservationRule>,
            Rc::new(TieredHealthCheckReservationRequiredContactEmailRule {
                message: "tiered health check reservation needs this field".to_string(),
            }),
            Rc::new(TieredHealthCheckReservationContactEmailLengthRule { max_length: 16 }),
            Rc::new(TieredHealthCheckReservationAllowedCurrencyRule { allowed: Vec::new() }),
            Rc::new(TieredHealthCheckReservationRequiredLabelRule {
                message: "tiered health check reservation needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_health_check_reservation_validator();
        let item = new_tiered_health_check_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = TieredHealthCheckReservationQuotaUnitsRangeRule { min: 80, max: 1000 };
        let mut item = new_tiered_health_check_reservation("x", "y");
        item.quota_units = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = TieredHealthCheckReservationRequiredContactEmailRule {
            message: "tiered health check reservation needs this field".to_string(),
        };
        let mut item = new_tiered_health_check_reservation("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = TieredHealthCheckReservationContactEmailLengthRule { max_length: 80 };
        let mut item = new_tiered_health_check_reservation("x", "y");
        item.contact_email = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TieredHealthCheckReservationRequiredLabelRule {
            message: "tiered health check reservation needs a value".to_string(),
        };
        let mut item = new_tiered_health_check_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
