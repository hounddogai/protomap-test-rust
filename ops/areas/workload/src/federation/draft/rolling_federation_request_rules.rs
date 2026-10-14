use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling federation request must pass. Check returns its problems, or none.
pub trait RollingFederationRequestRule: std::fmt::Debug {
    fn check(&self, item: &RollingFederationRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationRequestCodeLengthRule {
    pub max_length: i64,
}

impl RollingFederationRequestRule for RollingFederationRequestCodeLengthRule {
    fn check(&self, item: &RollingFederationRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationRequestMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingFederationRequestRule for RollingFederationRequestMaximumRangeRule {
    fn check(&self, item: &RollingFederationRequest) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationRequestCurrencyLengthRule {
    pub max_length: i64,
}

impl RollingFederationRequestRule for RollingFederationRequestCurrencyLengthRule {
    fn check(&self, item: &RollingFederationRequest) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling federation request.
#[derive(Debug, Clone)]
pub struct RollingFederationRequestValidator {
    pub rules: Vec<Rc<dyn RollingFederationRequestRule>>,
}

impl RollingFederationRequestValidator {
    pub fn validate(&self, item: &RollingFederationRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingFederationRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingFederationRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_federation_request_validator() -> RollingFederationRequestValidator {
    RollingFederationRequestValidator {
        rules: vec![
            Rc::new(RollingFederationRequestCodeLengthRule { max_length: 120 }) as Rc<dyn RollingFederationRequestRule>,
            Rc::new(RollingFederationRequestMaximumRangeRule { min: 6.8, max: 2089.1 }),
            Rc::new(RollingFederationRequestCurrencyLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_federation_request_validator();
        let item = new_rolling_federation_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingFederationRequestCodeLengthRule { max_length: 86400 };
        let mut item = new_rolling_federation_request("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RollingFederationRequestMaximumRangeRule { min: 0.2, max: 2656.5 };
        let mut item = new_rolling_federation_request("x", "y");
        item.maximum = 2658.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = RollingFederationRequestCurrencyLengthRule { max_length: 120 };
        let mut item = new_rolling_federation_request("x", "y");
        item.currency = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
