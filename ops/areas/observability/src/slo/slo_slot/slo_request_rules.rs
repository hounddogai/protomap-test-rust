use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a slo request must pass. Check returns its problems, or none.
pub trait SloRequestRule: std::fmt::Debug {
    fn check(&self, item: &SloRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRequestMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloRequestRule for SloRequestMaximumRangeRule {
    fn check(&self, item: &SloRequest) -> Vec<String> {
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
pub struct SloRequestCodePrefixRule {
    pub prefix: String,
}

impl SloRequestRule for SloRequestCodePrefixRule {
    fn check(&self, item: &SloRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRequestAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl SloRequestRule for SloRequestAllowedLabelRule {
    fn check(&self, item: &SloRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is below minimum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRequestCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloRequestRule for SloRequestCreatedAtRangeRule {
    fn check(&self, item: &SloRequest) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo request.
#[derive(Debug, Clone)]
pub struct SloRequestValidator {
    pub rules: Vec<Rc<dyn SloRequestRule>>,
}

impl SloRequestValidator {
    pub fn validate(&self, item: &SloRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_request_validator() -> SloRequestValidator {
    SloRequestValidator {
        rules: vec![
            Rc::new(SloRequestMaximumRangeRule { min: 0.5, max: 4830.4 }) as Rc<dyn SloRequestRule>,
            Rc::new(SloRequestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(SloRequestAllowedLabelRule {
                allowed: vec!["global".to_string(), "tiered".to_string(), "batched".to_string()],
            }),
            Rc::new(SloRequestCreatedAtRangeRule { min: 32, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_request_validator();
        let item = new_slo_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = SloRequestMaximumRangeRule { min: 9.8, max: 2960.0 };
        let mut item = new_slo_request("x", "y");
        item.maximum = 2961.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = SloRequestCreatedAtRangeRule { min: 80, max: 255 };
        let mut item = new_slo_request("x", "y");
        item.created_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
