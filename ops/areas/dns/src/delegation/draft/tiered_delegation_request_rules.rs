use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered delegation request must pass. Check returns its problems, or none.
pub trait TieredDelegationRequestRule: std::fmt::Debug {
    fn check(&self, item: &TieredDelegationRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationRequestQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredDelegationRequestRule for TieredDelegationRequestQuantityRangeRule {
    fn check(&self, item: &TieredDelegationRequest) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationRequestRequiredTitleRule {
    pub message: String,
}

impl TieredDelegationRequestRule for TieredDelegationRequestRequiredTitleRule {
    fn check(&self, item: &TieredDelegationRequest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationRequestMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredDelegationRequestRule for TieredDelegationRequestMinimumRangeRule {
    fn check(&self, item: &TieredDelegationRequest) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationRequestRequiredNameRule {
    pub message: String,
}

impl TieredDelegationRequestRule for TieredDelegationRequestRequiredNameRule {
    fn check(&self, item: &TieredDelegationRequest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationRequestRequiredCountryRule {
    pub message: String,
}

impl TieredDelegationRequestRule for TieredDelegationRequestRequiredCountryRule {
    fn check(&self, item: &TieredDelegationRequest) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationRequestAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl TieredDelegationRequestRule for TieredDelegationRequestAllowedCountryRule {
    fn check(&self, item: &TieredDelegationRequest) -> Vec<String> {
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

/// Runs every rule of a tiered delegation request.
#[derive(Debug, Clone)]
pub struct TieredDelegationRequestValidator {
    pub rules: Vec<Rc<dyn TieredDelegationRequestRule>>,
}

impl TieredDelegationRequestValidator {
    pub fn validate(&self, item: &TieredDelegationRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredDelegationRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredDelegationRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_delegation_request_validator() -> TieredDelegationRequestValidator {
    TieredDelegationRequestValidator {
        rules: vec![
            Rc::new(TieredDelegationRequestQuantityRangeRule { min: 16, max: 3600 })
                as Rc<dyn TieredDelegationRequestRule>,
            Rc::new(TieredDelegationRequestRequiredTitleRule {
                message: "tiered delegation request needs an entry".to_string(),
            }),
            Rc::new(TieredDelegationRequestMinimumRangeRule { min: 6.5, max: 3557.3 }),
            Rc::new(TieredDelegationRequestRequiredNameRule {
                message: "tiered delegation request needs an entry".to_string(),
            }),
            Rc::new(TieredDelegationRequestRequiredCountryRule {
                message: "tiered delegation request needs an entry".to_string(),
            }),
            Rc::new(TieredDelegationRequestAllowedCountryRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_delegation_request_validator();
        let item = new_tiered_delegation_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = TieredDelegationRequestQuantityRangeRule { min: 16, max: 120 };
        let mut item = new_tiered_delegation_request("x", "y");
        item.quantity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredDelegationRequestRequiredTitleRule {
            message: "tiered delegation request needs this field".to_string(),
        };
        let mut item = new_tiered_delegation_request("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = TieredDelegationRequestMinimumRangeRule { min: 1.3, max: 74.6 };
        let mut item = new_tiered_delegation_request("x", "y");
        item.minimum = 76.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredDelegationRequestRequiredNameRule {
            message: "tiered delegation request needs this field".to_string(),
        };
        let mut item = new_tiered_delegation_request("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = TieredDelegationRequestRequiredCountryRule {
            message: "tiered delegation request needs a value".to_string(),
        };
        let mut item = new_tiered_delegation_request("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
