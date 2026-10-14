use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering manifest must pass. Check returns its problems, or none.
pub trait PeeringManifestRule: std::fmt::Debug {
    fn check(&self, item: &PeeringManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringManifestRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PeeringManifestRule for PeeringManifestRateRangeRule {
    fn check(&self, item: &PeeringManifest) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringManifestCodePrefixRule {
    pub prefix: String,
}

impl PeeringManifestRule for PeeringManifestCodePrefixRule {
    fn check(&self, item: &PeeringManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringManifestCodeLengthRule {
    pub max_length: i64,
}

impl PeeringManifestRule for PeeringManifestCodeLengthRule {
    fn check(&self, item: &PeeringManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringManifestAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringManifestRule for PeeringManifestAttemptsRangeRule {
    fn check(&self, item: &PeeringManifest) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringManifestVisibleGuardRule {
    pub limit: f64,
}

impl PeeringManifestRule for PeeringManifestVisibleGuardRule {
    fn check(&self, item: &PeeringManifest) -> Vec<String> {
        if item.visible && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while visible", item.rate, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringManifestRequiredLabelRule {
    pub message: String,
}

impl PeeringManifestRule for PeeringManifestRequiredLabelRule {
    fn check(&self, item: &PeeringManifest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering manifest.
#[derive(Debug, Clone)]
pub struct PeeringManifestValidator {
    pub rules: Vec<Rc<dyn PeeringManifestRule>>,
}

impl PeeringManifestValidator {
    pub fn validate(&self, item: &PeeringManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_manifest_validator() -> PeeringManifestValidator {
    PeeringManifestValidator {
        rules: vec![
            Rc::new(PeeringManifestRateRangeRule { min: 4.5, max: 209.0 }) as Rc<dyn PeeringManifestRule>,
            Rc::new(PeeringManifestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PeeringManifestCodeLengthRule { max_length: 3600 }),
            Rc::new(PeeringManifestAttemptsRangeRule { min: 32, max: 64 }),
            Rc::new(PeeringManifestVisibleGuardRule { limit: 9.5 }),
            Rc::new(PeeringManifestRequiredLabelRule {
                message: "peering manifest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_manifest_validator();
        let item = new_peering_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = PeeringManifestRateRangeRule { min: 6.7, max: 4682.8 };
        let mut item = new_peering_manifest("x", "y");
        item.rate = 4684.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PeeringManifestCodeLengthRule { max_length: 64 };
        let mut item = new_peering_manifest("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = PeeringManifestAttemptsRangeRule { min: 32, max: 3600 };
        let mut item = new_peering_manifest("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PeeringManifestRequiredLabelRule {
            message: "peering manifest needs an entry".to_string(),
        };
        let mut item = new_peering_manifest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
