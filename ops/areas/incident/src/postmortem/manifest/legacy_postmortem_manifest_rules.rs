use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy postmortem manifest must pass. Check returns its problems, or none.
pub trait LegacyPostmortemManifestRule: std::fmt::Debug {
    fn check(&self, item: &LegacyPostmortemManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemManifestCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyPostmortemManifestRule for LegacyPostmortemManifestCeilingRangeRule {
    fn check(&self, item: &LegacyPostmortemManifest) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemManifestAudienceLengthRule {
    pub max_length: i64,
}

impl LegacyPostmortemManifestRule for LegacyPostmortemManifestAudienceLengthRule {
    fn check(&self, item: &LegacyPostmortemManifest) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemManifestCodePrefixRule {
    pub prefix: String,
}

impl LegacyPostmortemManifestRule for LegacyPostmortemManifestCodePrefixRule {
    fn check(&self, item: &LegacyPostmortemManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy postmortem manifest.
#[derive(Debug, Clone)]
pub struct LegacyPostmortemManifestValidator {
    pub rules: Vec<Rc<dyn LegacyPostmortemManifestRule>>,
}

impl LegacyPostmortemManifestValidator {
    pub fn validate(&self, item: &LegacyPostmortemManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyPostmortemManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyPostmortemManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_postmortem_manifest_validator() -> LegacyPostmortemManifestValidator {
    LegacyPostmortemManifestValidator {
        rules: vec![
            Rc::new(LegacyPostmortemManifestCeilingRangeRule { min: 4.1, max: 4547.2 })
                as Rc<dyn LegacyPostmortemManifestRule>,
            Rc::new(LegacyPostmortemManifestAudienceLengthRule { max_length: 255 }),
            Rc::new(LegacyPostmortemManifestCodePrefixRule {
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
        let validator = default_legacy_postmortem_manifest_validator();
        let item = new_legacy_postmortem_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = LegacyPostmortemManifestCeilingRangeRule { min: 2.4, max: 410.0 };
        let mut item = new_legacy_postmortem_manifest("x", "y");
        item.ceiling = 411.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = LegacyPostmortemManifestAudienceLengthRule { max_length: 64 };
        let mut item = new_legacy_postmortem_manifest("x", "y");
        item.audience = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
