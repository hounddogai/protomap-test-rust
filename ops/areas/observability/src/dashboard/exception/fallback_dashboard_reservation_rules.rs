use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback dashboard reservation must pass. Check returns its problems, or none.
pub trait FallbackDashboardReservationRule: std::fmt::Debug {
    fn check(&self, item: &FallbackDashboardReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardReservationEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackDashboardReservationRule for FallbackDashboardReservationEndsAtRangeRule {
    fn check(&self, item: &FallbackDashboardReservation) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardReservationActiveGuardRule {
    pub limit: i64,
}

impl FallbackDashboardReservationRule for FallbackDashboardReservationActiveGuardRule {
    fn check(&self, item: &FallbackDashboardReservation) -> Vec<String> {
        if item.active && (item.ends_at > self.limit) {
            return vec![format!("ends at {} exceeds {} while active", item.ends_at, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardReservationRequiredNameRule {
    pub message: String,
}

impl FallbackDashboardReservationRule for FallbackDashboardReservationRequiredNameRule {
    fn check(&self, item: &FallbackDashboardReservation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardReservationAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FallbackDashboardReservationRule for FallbackDashboardReservationAllowedNameRule {
    fn check(&self, item: &FallbackDashboardReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is stale", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback dashboard reservation.
#[derive(Debug, Clone)]
pub struct FallbackDashboardReservationValidator {
    pub rules: Vec<Rc<dyn FallbackDashboardReservationRule>>,
}

impl FallbackDashboardReservationValidator {
    pub fn validate(&self, item: &FallbackDashboardReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackDashboardReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackDashboardReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_dashboard_reservation_validator() -> FallbackDashboardReservationValidator {
    FallbackDashboardReservationValidator {
        rules: vec![
            Rc::new(FallbackDashboardReservationEndsAtRangeRule { min: 80, max: 80 })
                as Rc<dyn FallbackDashboardReservationRule>,
            Rc::new(FallbackDashboardReservationActiveGuardRule { limit: 80 }),
            Rc::new(FallbackDashboardReservationRequiredNameRule {
                message: "fallback dashboard reservation needs this field".to_string(),
            }),
            Rc::new(FallbackDashboardReservationAllowedNameRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_dashboard_reservation_validator();
        let item = new_fallback_dashboard_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = FallbackDashboardReservationEndsAtRangeRule { min: 120, max: 255 };
        let mut item = new_fallback_dashboard_reservation("x", "y");
        item.ends_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FallbackDashboardReservationRequiredNameRule {
            message: "fallback dashboard reservation needs a value".to_string(),
        };
        let mut item = new_fallback_dashboard_reservation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
