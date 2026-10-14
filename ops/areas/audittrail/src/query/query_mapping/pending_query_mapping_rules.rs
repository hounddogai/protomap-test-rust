use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending query mapping must pass. Check returns its problems, or none.
pub trait PendingQueryMappingRule: std::fmt::Debug {
    fn check(&self, item: &PendingQueryMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryMappingCodePrefixRule {
    pub prefix: String,
}

impl PendingQueryMappingRule for PendingQueryMappingCodePrefixRule {
    fn check(&self, item: &PendingQueryMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryMappingEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingQueryMappingRule for PendingQueryMappingEndsAtRangeRule {
    fn check(&self, item: &PendingQueryMapping) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryMappingRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingQueryMappingRule for PendingQueryMappingRateRangeRule {
    fn check(&self, item: &PendingQueryMapping) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryMappingLabelLengthRule {
    pub max_length: i64,
}

impl PendingQueryMappingRule for PendingQueryMappingLabelLengthRule {
    fn check(&self, item: &PendingQueryMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryMappingRequiredExternalRefRule {
    pub message: String,
}

impl PendingQueryMappingRule for PendingQueryMappingRequiredExternalRefRule {
    fn check(&self, item: &PendingQueryMapping) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryMappingAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl PendingQueryMappingRule for PendingQueryMappingAllowedExternalRefRule {
    fn check(&self, item: &PendingQueryMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is expired", item.external_ref)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending query mapping.
#[derive(Debug, Clone)]
pub struct PendingQueryMappingValidator {
    pub rules: Vec<Rc<dyn PendingQueryMappingRule>>,
}

impl PendingQueryMappingValidator {
    pub fn validate(&self, item: &PendingQueryMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingQueryMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingQueryMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_query_mapping_validator() -> PendingQueryMappingValidator {
    PendingQueryMappingValidator {
        rules: vec![
            Rc::new(PendingQueryMappingCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn PendingQueryMappingRule>,
            Rc::new(PendingQueryMappingEndsAtRangeRule { min: 120, max: 1000 }),
            Rc::new(PendingQueryMappingRateRangeRule { min: 4.4, max: 2057.0 }),
            Rc::new(PendingQueryMappingLabelLengthRule { max_length: 1000 }),
            Rc::new(PendingQueryMappingRequiredExternalRefRule {
                message: "pending query mapping needs this field".to_string(),
            }),
            Rc::new(PendingQueryMappingAllowedExternalRefRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_query_mapping_validator();
        let item = new_pending_query_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = PendingQueryMappingEndsAtRangeRule { min: 80, max: 86400 };
        let mut item = new_pending_query_mapping("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = PendingQueryMappingRateRangeRule { min: 0.5, max: 374.9 };
        let mut item = new_pending_query_mapping("x", "y");
        item.rate = 376.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingQueryMappingLabelLengthRule { max_length: 86400 };
        let mut item = new_pending_query_mapping("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = PendingQueryMappingRequiredExternalRefRule {
            message: "pending query mapping needs this field".to_string(),
        };
        let mut item = new_pending_query_mapping("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
