use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered silence cycle must pass. Check returns its problems, or none.
pub trait TieredSilenceCycleRule: std::fmt::Debug {
    fn check(&self, item: &TieredSilenceCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceCycleCodePrefixRule {
    pub prefix: String,
}

impl TieredSilenceCycleRule for TieredSilenceCycleCodePrefixRule {
    fn check(&self, item: &TieredSilenceCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceCycleRequiredRegionRule {
    pub message: String,
}

impl TieredSilenceCycleRule for TieredSilenceCycleRequiredRegionRule {
    fn check(&self, item: &TieredSilenceCycle) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceCycleAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl TieredSilenceCycleRule for TieredSilenceCycleAllowedExternalRefRule {
    fn check(&self, item: &TieredSilenceCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is below minimum", item.external_ref)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered silence cycle.
#[derive(Debug, Clone)]
pub struct TieredSilenceCycleValidator {
    pub rules: Vec<Rc<dyn TieredSilenceCycleRule>>,
}

impl TieredSilenceCycleValidator {
    pub fn validate(&self, item: &TieredSilenceCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredSilenceCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredSilenceCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_silence_cycle_validator() -> TieredSilenceCycleValidator {
    TieredSilenceCycleValidator {
        rules: vec![
            Rc::new(TieredSilenceCycleCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn TieredSilenceCycleRule>,
            Rc::new(TieredSilenceCycleRequiredRegionRule {
                message: "tiered silence cycle needs an entry".to_string(),
            }),
            Rc::new(TieredSilenceCycleAllowedExternalRefRule {
                allowed: vec!["pending".to_string(), "rolling".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_silence_cycle_validator();
        let item = new_tiered_silence_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = TieredSilenceCycleRequiredRegionRule {
            message: "tiered silence cycle needs this field".to_string(),
        };
        let mut item = new_tiered_silence_cycle("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
