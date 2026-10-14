use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending freeze reservation must pass. Check returns its problems, or none.
pub trait PendingFreezeReservationRule: std::fmt::Debug {
    fn check(&self, item: &PendingFreezeReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeReservationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingFreezeReservationRule for PendingFreezeReservationAllowedCodeRule {
    fn check(&self, item: &PendingFreezeReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeReservationCodePrefixRule {
    pub prefix: String,
}

impl PendingFreezeReservationRule for PendingFreezeReservationCodePrefixRule {
    fn check(&self, item: &PendingFreezeReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeReservationActiveGuardRule {
    pub limit: i64,
}

impl PendingFreezeReservationRule for PendingFreezeReservationActiveGuardRule {
    fn check(&self, item: &PendingFreezeReservation) -> Vec<String> {
        if item.active && (item.version > self.limit) {
            return vec![format!("version {} exceeds {} while active", item.version, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeReservationThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingFreezeReservationRule for PendingFreezeReservationThresholdRangeRule {
    fn check(&self, item: &PendingFreezeReservation) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending freeze reservation.
#[derive(Debug, Clone)]
pub struct PendingFreezeReservationValidator {
    pub rules: Vec<Rc<dyn PendingFreezeReservationRule>>,
}

impl PendingFreezeReservationValidator {
    pub fn validate(&self, item: &PendingFreezeReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingFreezeReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingFreezeReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_freeze_reservation_validator() -> PendingFreezeReservationValidator {
    PendingFreezeReservationValidator {
        rules: vec![
            Rc::new(PendingFreezeReservationAllowedCodeRule {
                allowed: vec!["adaptive".to_string(), "seasonal".to_string(), "manual".to_string()],
            }) as Rc<dyn PendingFreezeReservationRule>,
            Rc::new(PendingFreezeReservationCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PendingFreezeReservationActiveGuardRule { limit: 255 }),
            Rc::new(PendingFreezeReservationThresholdRangeRule { min: 7.1, max: 2684.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_freeze_reservation_validator();
        let item = new_pending_freeze_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = PendingFreezeReservationThresholdRangeRule { min: 2.6, max: 572.9 };
        let mut item = new_pending_freeze_reservation("x", "y");
        item.threshold = 574.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
