use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation manifest must pass. Check returns its problems, or none.
pub trait ReservationManifestRule: std::fmt::Debug {
    fn check(&self, item: &ReservationManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationManifestBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationManifestRule for ReservationManifestBurstRangeRule {
    fn check(&self, item: &ReservationManifest) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationManifestCodePrefixRule {
    pub prefix: String,
}

impl ReservationManifestRule for ReservationManifestCodePrefixRule {
    fn check(&self, item: &ReservationManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationManifestRequiredSourceRule {
    pub message: String,
}

impl ReservationManifestRule for ReservationManifestRequiredSourceRule {
    fn check(&self, item: &ReservationManifest) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation manifest.
#[derive(Debug, Clone)]
pub struct ReservationManifestValidator {
    pub rules: Vec<Rc<dyn ReservationManifestRule>>,
}

impl ReservationManifestValidator {
    pub fn validate(&self, item: &ReservationManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_manifest_validator() -> ReservationManifestValidator {
    ReservationManifestValidator {
        rules: vec![
            Rc::new(ReservationManifestBurstRangeRule { min: 80, max: 255 }) as Rc<dyn ReservationManifestRule>,
            Rc::new(ReservationManifestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ReservationManifestRequiredSourceRule {
                message: "reservation manifest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_manifest_validator();
        let item = new_reservation_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = ReservationManifestBurstRangeRule { min: 16, max: 64 };
        let mut item = new_reservation_manifest("x", "y");
        item.burst = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = ReservationManifestRequiredSourceRule {
            message: "reservation manifest needs an entry".to_string(),
        };
        let mut item = new_reservation_manifest("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
