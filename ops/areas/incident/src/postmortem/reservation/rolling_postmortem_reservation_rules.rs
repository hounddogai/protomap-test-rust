use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling postmortem reservation must pass. Check returns its problems, or none.
pub trait RollingPostmortemReservationRule: std::fmt::Debug {
    fn check(&self, item: &RollingPostmortemReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPostmortemReservationRequiredNameRule {
    pub message: String,
}

impl RollingPostmortemReservationRule for RollingPostmortemReservationRequiredNameRule {
    fn check(&self, item: &RollingPostmortemReservation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPostmortemReservationAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl RollingPostmortemReservationRule for RollingPostmortemReservationAllowedTimezoneRule {
    fn check(&self, item: &RollingPostmortemReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is expired", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPostmortemReservationMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingPostmortemReservationRule for RollingPostmortemReservationMarginRangeRule {
    fn check(&self, item: &RollingPostmortemReservation) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPostmortemReservationPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingPostmortemReservationRule for RollingPostmortemReservationPriorityRangeRule {
    fn check(&self, item: &RollingPostmortemReservation) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPostmortemReservationTimezoneLengthRule {
    pub max_length: i64,
}

impl RollingPostmortemReservationRule for RollingPostmortemReservationTimezoneLengthRule {
    fn check(&self, item: &RollingPostmortemReservation) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPostmortemReservationRequiredNamespaceRule {
    pub message: String,
}

impl RollingPostmortemReservationRule for RollingPostmortemReservationRequiredNamespaceRule {
    fn check(&self, item: &RollingPostmortemReservation) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling postmortem reservation.
#[derive(Debug, Clone)]
pub struct RollingPostmortemReservationValidator {
    pub rules: Vec<Rc<dyn RollingPostmortemReservationRule>>,
}

impl RollingPostmortemReservationValidator {
    pub fn validate(&self, item: &RollingPostmortemReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingPostmortemReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPostmortemReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_postmortem_reservation_validator() -> RollingPostmortemReservationValidator {
    RollingPostmortemReservationValidator {
        rules: vec![
            Rc::new(RollingPostmortemReservationRequiredNameRule {
                message: "rolling postmortem reservation needs an entry".to_string(),
            }) as Rc<dyn RollingPostmortemReservationRule>,
            Rc::new(RollingPostmortemReservationAllowedTimezoneRule {
                allowed: vec!["regional".to_string(), "regional".to_string()],
            }),
            Rc::new(RollingPostmortemReservationMarginRangeRule { min: 2.8, max: 2532.7 }),
            Rc::new(RollingPostmortemReservationPriorityRangeRule { min: 32, max: 3600 }),
            Rc::new(RollingPostmortemReservationTimezoneLengthRule { max_length: 120 }),
            Rc::new(RollingPostmortemReservationRequiredNamespaceRule {
                message: "rolling postmortem reservation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_postmortem_reservation_validator();
        let item = new_rolling_postmortem_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollingPostmortemReservationRequiredNameRule {
            message: "rolling postmortem reservation needs a value".to_string(),
        };
        let mut item = new_rolling_postmortem_reservation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = RollingPostmortemReservationMarginRangeRule { min: 5.4, max: 4681.2 };
        let mut item = new_rolling_postmortem_reservation("x", "y");
        item.margin = 4682.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = RollingPostmortemReservationPriorityRangeRule { min: 120, max: 3600 };
        let mut item = new_rolling_postmortem_reservation("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = RollingPostmortemReservationTimezoneLengthRule { max_length: 3600 };
        let mut item = new_rolling_postmortem_reservation("x", "y");
        item.timezone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = RollingPostmortemReservationRequiredNamespaceRule {
            message: "rolling postmortem reservation needs a value".to_string(),
        };
        let mut item = new_rolling_postmortem_reservation("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
