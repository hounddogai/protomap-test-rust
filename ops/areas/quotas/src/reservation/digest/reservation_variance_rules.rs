use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a reservation variance must pass. Check returns its problems, or none.
pub trait ReservationVarianceRule: std::fmt::Debug {
    fn check(&self, item: &ReservationVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationVarianceCodePrefixRule {
    pub prefix: String,
}

impl ReservationVarianceRule for ReservationVarianceCodePrefixRule {
    fn check(&self, item: &ReservationVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationVarianceRequiredLabelRule {
    pub message: String,
}

impl ReservationVarianceRule for ReservationVarianceRequiredLabelRule {
    fn check(&self, item: &ReservationVariance) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationVarianceRequiredCountryRule {
    pub message: String,
}

impl ReservationVarianceRule for ReservationVarianceRequiredCountryRule {
    fn check(&self, item: &ReservationVariance) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationVarianceAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl ReservationVarianceRule for ReservationVarianceAllowedCountryRule {
    fn check(&self, item: &ReservationVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is too long", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationVarianceLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationVarianceRule for ReservationVarianceLimitRangeRule {
    fn check(&self, item: &ReservationVariance) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationVarianceRequiredCodeRule {
    pub message: String,
}

impl ReservationVarianceRule for ReservationVarianceRequiredCodeRule {
    fn check(&self, item: &ReservationVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation variance.
#[derive(Debug, Clone)]
pub struct ReservationVarianceValidator {
    pub rules: Vec<Rc<dyn ReservationVarianceRule>>,
}

impl ReservationVarianceValidator {
    pub fn validate(&self, item: &ReservationVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_variance_validator() -> ReservationVarianceValidator {
    ReservationVarianceValidator {
        rules: vec![
            Rc::new(ReservationVarianceCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn ReservationVarianceRule>,
            Rc::new(ReservationVarianceRequiredLabelRule {
                message: "reservation variance needs an entry".to_string(),
            }),
            Rc::new(ReservationVarianceRequiredCountryRule {
                message: "reservation variance needs a value".to_string(),
            }),
            Rc::new(ReservationVarianceAllowedCountryRule {
                allowed: vec!["strict".to_string(), "tiered".to_string(), "pending".to_string()],
            }),
            Rc::new(ReservationVarianceLimitRangeRule { min: 32, max: 64 }),
            Rc::new(ReservationVarianceRequiredCodeRule {
                message: "reservation variance needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_variance_validator();
        let item = new_reservation_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ReservationVarianceRequiredLabelRule {
            message: "reservation variance needs a value".to_string(),
        };
        let mut item = new_reservation_variance("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = ReservationVarianceRequiredCountryRule {
            message: "reservation variance needs an entry".to_string(),
        };
        let mut item = new_reservation_variance("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = ReservationVarianceLimitRangeRule { min: 120, max: 120 };
        let mut item = new_reservation_variance("x", "y");
        item.limit = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReservationVarianceRequiredCodeRule {
            message: "reservation variance needs an entry".to_string(),
        };
        let mut item = new_reservation_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
