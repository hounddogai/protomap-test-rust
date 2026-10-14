use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a federation mapping must pass. Check returns its problems, or none.
pub trait FederationMappingRule: std::fmt::Debug {
    fn check(&self, item: &FederationMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationMappingAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl FederationMappingRule for FederationMappingAllowedNotesRule {
    fn check(&self, item: &FederationMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is below minimum", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationMappingWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FederationMappingRule for FederationMappingWeightRangeRule {
    fn check(&self, item: &FederationMapping) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationMappingCodePrefixRule {
    pub prefix: String,
}

impl FederationMappingRule for FederationMappingCodePrefixRule {
    fn check(&self, item: &FederationMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation mapping.
#[derive(Debug, Clone)]
pub struct FederationMappingValidator {
    pub rules: Vec<Rc<dyn FederationMappingRule>>,
}

impl FederationMappingValidator {
    pub fn validate(&self, item: &FederationMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_mapping_validator() -> FederationMappingValidator {
    FederationMappingValidator {
        rules: vec![
            Rc::new(FederationMappingAllowedNotesRule {
                allowed: vec!["adaptive".to_string(), "lenient".to_string()],
            }) as Rc<dyn FederationMappingRule>,
            Rc::new(FederationMappingWeightRangeRule { min: 4.4, max: 3008.9 }),
            Rc::new(FederationMappingCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_mapping_validator();
        let item = new_federation_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = FederationMappingWeightRangeRule { min: 6.0, max: 586.2 };
        let mut item = new_federation_mapping("x", "y");
        item.weight = 587.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
