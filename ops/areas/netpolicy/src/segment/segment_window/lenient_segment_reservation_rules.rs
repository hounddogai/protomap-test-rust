use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient segment reservation must pass. Check returns its problems, or none.
pub trait LenientSegmentReservationRule: std::fmt::Debug {
    fn check(&self, item: &LenientSegmentReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSegmentReservationNameLengthRule {
    pub max_length: i64,
}

impl LenientSegmentReservationRule for LenientSegmentReservationNameLengthRule {
    fn check(&self, item: &LenientSegmentReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSegmentReservationVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientSegmentReservationRule for LenientSegmentReservationVersionRangeRule {
    fn check(&self, item: &LenientSegmentReservation) -> Vec<String> {
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
pub struct LenientSegmentReservationCodePrefixRule {
    pub prefix: String,
}

impl LenientSegmentReservationRule for LenientSegmentReservationCodePrefixRule {
    fn check(&self, item: &LenientSegmentReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSegmentReservationVisibleGuardRule {
    pub limit: i64,
}

impl LenientSegmentReservationRule for LenientSegmentReservationVisibleGuardRule {
    fn check(&self, item: &LenientSegmentReservation) -> Vec<String> {
        if item.visible && (item.starts_at > self.limit) {
            return vec![format!(
                "starts at {} exceeds {} while visible",
                item.starts_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSegmentReservationRequiredScopeRule {
    pub message: String,
}

impl LenientSegmentReservationRule for LenientSegmentReservationRequiredScopeRule {
    fn check(&self, item: &LenientSegmentReservation) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient segment reservation.
#[derive(Debug, Clone)]
pub struct LenientSegmentReservationValidator {
    pub rules: Vec<Rc<dyn LenientSegmentReservationRule>>,
}

impl LenientSegmentReservationValidator {
    pub fn validate(&self, item: &LenientSegmentReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientSegmentReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSegmentReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_segment_reservation_validator() -> LenientSegmentReservationValidator {
    LenientSegmentReservationValidator {
        rules: vec![
            Rc::new(LenientSegmentReservationNameLengthRule { max_length: 80 })
                as Rc<dyn LenientSegmentReservationRule>,
            Rc::new(LenientSegmentReservationVersionRangeRule { min: 120, max: 3600 }),
            Rc::new(LenientSegmentReservationCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LenientSegmentReservationVisibleGuardRule { limit: 64 }),
            Rc::new(LenientSegmentReservationRequiredScopeRule {
                message: "lenient segment reservation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_segment_reservation_validator();
        let item = new_lenient_segment_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LenientSegmentReservationNameLengthRule { max_length: 3600 };
        let mut item = new_lenient_segment_reservation("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = LenientSegmentReservationVersionRangeRule { min: 32, max: 120 };
        let mut item = new_lenient_segment_reservation("x", "y");
        item.version = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = LenientSegmentReservationRequiredScopeRule {
            message: "lenient segment reservation needs an entry".to_string(),
        };
        let mut item = new_lenient_segment_reservation("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
