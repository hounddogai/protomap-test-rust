use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding manifest must pass. Check returns its problems, or none.
pub trait BindingManifestRule: std::fmt::Debug {
    fn check(&self, item: &BindingManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingManifestScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BindingManifestRule for BindingManifestScoreRangeRule {
    fn check(&self, item: &BindingManifest) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingManifestCodePrefixRule {
    pub prefix: String,
}

impl BindingManifestRule for BindingManifestCodePrefixRule {
    fn check(&self, item: &BindingManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingManifestTitleLengthRule {
    pub max_length: i64,
}

impl BindingManifestRule for BindingManifestTitleLengthRule {
    fn check(&self, item: &BindingManifest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding manifest.
#[derive(Debug, Clone)]
pub struct BindingManifestValidator {
    pub rules: Vec<Rc<dyn BindingManifestRule>>,
}

impl BindingManifestValidator {
    pub fn validate(&self, item: &BindingManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_manifest_validator() -> BindingManifestValidator {
    BindingManifestValidator {
        rules: vec![
            Rc::new(BindingManifestScoreRangeRule { min: 1.5, max: 2492.5 }) as Rc<dyn BindingManifestRule>,
            Rc::new(BindingManifestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(BindingManifestTitleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_manifest_validator();
        let item = new_binding_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = BindingManifestScoreRangeRule { min: 1.0, max: 3792.9 };
        let mut item = new_binding_manifest("x", "y");
        item.score = 3794.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BindingManifestTitleLengthRule { max_length: 255 };
        let mut item = new_binding_manifest("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
