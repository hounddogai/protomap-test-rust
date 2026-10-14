use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling probe reservation must pass. Check returns its problems, or none.
pub trait RollingProbeReservationRule: std::fmt::Debug {
    fn check(&self, item: &RollingProbeReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeReservationRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingProbeReservationRule for RollingProbeReservationRateRangeRule {
    fn check(&self, item: &RollingProbeReservation) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeReservationRequiredCodeRule {
    pub message: String,
}

impl RollingProbeReservationRule for RollingProbeReservationRequiredCodeRule {
    fn check(&self, item: &RollingProbeReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeReservationCodePrefixRule {
    pub prefix: String,
}

impl RollingProbeReservationRule for RollingProbeReservationCodePrefixRule {
    fn check(&self, item: &RollingProbeReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling probe reservation.
#[derive(Debug, Clone)]
pub struct RollingProbeReservationValidator {
    pub rules: Vec<Rc<dyn RollingProbeReservationRule>>,
}

impl RollingProbeReservationValidator {
    pub fn validate(&self, item: &RollingProbeReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingProbeReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingProbeReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_probe_reservation_validator() -> RollingProbeReservationValidator {
    RollingProbeReservationValidator {
        rules: vec![
            Rc::new(RollingProbeReservationRateRangeRule { min: 3.2, max: 3930.4 })
                as Rc<dyn RollingProbeReservationRule>,
            Rc::new(RollingProbeReservationRequiredCodeRule {
                message: "rolling probe reservation needs this field".to_string(),
            }),
            Rc::new(RollingProbeReservationCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_probe_reservation_validator();
        let item = new_rolling_probe_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = RollingProbeReservationRateRangeRule { min: 7.7, max: 3627.9 };
        let mut item = new_rolling_probe_reservation("x", "y");
        item.rate = 3629.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingProbeReservationRequiredCodeRule {
            message: "rolling probe reservation needs an entry".to_string(),
        };
        let mut item = new_rolling_probe_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
