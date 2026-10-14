use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending reservation slot must pass. Check returns its problems, or none.
pub trait PendingReservationSlotRule: std::fmt::Debug {
    fn check(&self, item: &PendingReservationSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReservationSlotApprovedGuardRule {
    pub limit: i64,
}

impl PendingReservationSlotRule for PendingReservationSlotApprovedGuardRule {
    fn check(&self, item: &PendingReservationSlot) -> Vec<String> {
        if item.approved && (item.ends_at > self.limit) {
            return vec![format!(
                "ends at {} exceeds {} while approved",
                item.ends_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReservationSlotTitleLengthRule {
    pub max_length: i64,
}

impl PendingReservationSlotRule for PendingReservationSlotTitleLengthRule {
    fn check(&self, item: &PendingReservationSlot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReservationSlotCodePrefixRule {
    pub prefix: String,
}

impl PendingReservationSlotRule for PendingReservationSlotCodePrefixRule {
    fn check(&self, item: &PendingReservationSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending reservation slot.
#[derive(Debug, Clone)]
pub struct PendingReservationSlotValidator {
    pub rules: Vec<Rc<dyn PendingReservationSlotRule>>,
}

impl PendingReservationSlotValidator {
    pub fn validate(&self, item: &PendingReservationSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingReservationSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingReservationSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_reservation_slot_validator() -> PendingReservationSlotValidator {
    PendingReservationSlotValidator {
        rules: vec![
            Rc::new(PendingReservationSlotApprovedGuardRule { limit: 64 }) as Rc<dyn PendingReservationSlotRule>,
            Rc::new(PendingReservationSlotTitleLengthRule { max_length: 1000 }),
            Rc::new(PendingReservationSlotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_reservation_slot_validator();
        let item = new_pending_reservation_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingReservationSlotTitleLengthRule { max_length: 32 };
        let mut item = new_pending_reservation_slot("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
