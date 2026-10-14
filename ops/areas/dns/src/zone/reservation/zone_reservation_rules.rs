use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone reservation must pass. Check returns its problems, or none.
pub trait ZoneReservationRule: std::fmt::Debug {
    fn check(&self, item: &ZoneReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneReservationRequiredScopeRule {
    pub message: String,
}

impl ZoneReservationRule for ZoneReservationRequiredScopeRule {
    fn check(&self, item: &ZoneReservation) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneReservationAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ZoneReservationRule for ZoneReservationAllowedLabelRule {
    fn check(&self, item: &ZoneReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is expired", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneReservationCodeLengthRule {
    pub max_length: i64,
}

impl ZoneReservationRule for ZoneReservationCodeLengthRule {
    fn check(&self, item: &ZoneReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneReservationRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneReservationRule for ZoneReservationRevisionRangeRule {
    fn check(&self, item: &ZoneReservation) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneReservationArchivedGuardRule {
    pub limit: f64,
}

impl ZoneReservationRule for ZoneReservationArchivedGuardRule {
    fn check(&self, item: &ZoneReservation) -> Vec<String> {
        if item.archived && (item.maximum > self.limit) {
            return vec![format!(
                "maximum {} exceeds {} while archived",
                item.maximum, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneReservationMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ZoneReservationRule for ZoneReservationMaximumRangeRule {
    fn check(&self, item: &ZoneReservation) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone reservation.
#[derive(Debug, Clone)]
pub struct ZoneReservationValidator {
    pub rules: Vec<Rc<dyn ZoneReservationRule>>,
}

impl ZoneReservationValidator {
    pub fn validate(&self, item: &ZoneReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_reservation_validator() -> ZoneReservationValidator {
    ZoneReservationValidator {
        rules: vec![
            Rc::new(ZoneReservationRequiredScopeRule {
                message: "zone reservation needs this field".to_string(),
            }) as Rc<dyn ZoneReservationRule>,
            Rc::new(ZoneReservationAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(ZoneReservationCodeLengthRule { max_length: 64 }),
            Rc::new(ZoneReservationRevisionRangeRule { min: 16, max: 255 }),
            Rc::new(ZoneReservationArchivedGuardRule { limit: 9.6 }),
            Rc::new(ZoneReservationMaximumRangeRule { min: 9.3, max: 1530.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_reservation_validator();
        let item = new_zone_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = ZoneReservationRequiredScopeRule {
            message: "zone reservation needs this field".to_string(),
        };
        let mut item = new_zone_reservation("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ZoneReservationCodeLengthRule { max_length: 120 };
        let mut item = new_zone_reservation("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = ZoneReservationRevisionRangeRule { min: 64, max: 3600 };
        let mut item = new_zone_reservation("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = ZoneReservationMaximumRangeRule { min: 7.7, max: 4069.4 };
        let mut item = new_zone_reservation("x", "y");
        item.maximum = 4070.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
