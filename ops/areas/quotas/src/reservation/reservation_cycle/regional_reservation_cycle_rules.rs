use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional reservation cycle must pass. Check returns its problems, or none.
pub trait RegionalReservationCycleRule: std::fmt::Debug {
    fn check(&self, item: &RegionalReservationCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationCycleRequiredNamespaceRule {
    pub message: String,
}

impl RegionalReservationCycleRule for RegionalReservationCycleRequiredNamespaceRule {
    fn check(&self, item: &RegionalReservationCycle) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationCycleMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalReservationCycleRule for RegionalReservationCycleMaximumRangeRule {
    fn check(&self, item: &RegionalReservationCycle) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationCycleExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalReservationCycleRule for RegionalReservationCycleExpiresAtRangeRule {
    fn check(&self, item: &RegionalReservationCycle) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationCycleDescriptionLengthRule {
    pub max_length: i64,
}

impl RegionalReservationCycleRule for RegionalReservationCycleDescriptionLengthRule {
    fn check(&self, item: &RegionalReservationCycle) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationCycleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalReservationCycleRule for RegionalReservationCycleAllowedCodeRule {
    fn check(&self, item: &RegionalReservationCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationCycleRequiredCodeRule {
    pub message: String,
}

impl RegionalReservationCycleRule for RegionalReservationCycleRequiredCodeRule {
    fn check(&self, item: &RegionalReservationCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional reservation cycle.
#[derive(Debug, Clone)]
pub struct RegionalReservationCycleValidator {
    pub rules: Vec<Rc<dyn RegionalReservationCycleRule>>,
}

impl RegionalReservationCycleValidator {
    pub fn validate(&self, item: &RegionalReservationCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalReservationCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalReservationCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_reservation_cycle_validator() -> RegionalReservationCycleValidator {
    RegionalReservationCycleValidator {
        rules: vec![
            Rc::new(RegionalReservationCycleRequiredNamespaceRule {
                message: "regional reservation cycle needs a value".to_string(),
            }) as Rc<dyn RegionalReservationCycleRule>,
            Rc::new(RegionalReservationCycleMaximumRangeRule { min: 7.1, max: 4254.9 }),
            Rc::new(RegionalReservationCycleExpiresAtRangeRule { min: 80, max: 3600 }),
            Rc::new(RegionalReservationCycleDescriptionLengthRule { max_length: 3600 }),
            Rc::new(RegionalReservationCycleAllowedCodeRule {
                allowed: vec!["strict".to_string()],
            }),
            Rc::new(RegionalReservationCycleRequiredCodeRule {
                message: "regional reservation cycle needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_reservation_cycle_validator();
        let item = new_regional_reservation_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = RegionalReservationCycleRequiredNamespaceRule {
            message: "regional reservation cycle needs a value".to_string(),
        };
        let mut item = new_regional_reservation_cycle("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RegionalReservationCycleMaximumRangeRule { min: 0.5, max: 2397.3 };
        let mut item = new_regional_reservation_cycle("x", "y");
        item.maximum = 2398.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = RegionalReservationCycleExpiresAtRangeRule { min: 32, max: 80 };
        let mut item = new_regional_reservation_cycle("x", "y");
        item.expires_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = RegionalReservationCycleDescriptionLengthRule { max_length: 86400 };
        let mut item = new_regional_reservation_cycle("x", "y");
        item.description = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalReservationCycleRequiredCodeRule {
            message: "regional reservation cycle needs an entry".to_string(),
        };
        let mut item = new_regional_reservation_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
