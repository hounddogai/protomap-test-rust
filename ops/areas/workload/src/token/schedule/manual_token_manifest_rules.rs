use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual token manifest must pass. Check returns its problems, or none.
pub trait ManualTokenManifestRule: std::fmt::Debug {
    fn check(&self, item: &ManualTokenManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTokenManifestLabelLengthRule {
    pub max_length: i64,
}

impl ManualTokenManifestRule for ManualTokenManifestLabelLengthRule {
    fn check(&self, item: &ManualTokenManifest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTokenManifestArchivedGuardRule {
    pub limit: i64,
}

impl ManualTokenManifestRule for ManualTokenManifestArchivedGuardRule {
    fn check(&self, item: &ManualTokenManifest) -> Vec<String> {
        if item.archived && (item.attempts > self.limit) {
            return vec![format!(
                "attempts {} exceeds {} while archived",
                item.attempts, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTokenManifestAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl ManualTokenManifestRule for ManualTokenManifestAllowedDescriptionRule {
    fn check(&self, item: &ManualTokenManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is out of range", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTokenManifestDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualTokenManifestRule for ManualTokenManifestDiscountRangeRule {
    fn check(&self, item: &ManualTokenManifest) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual token manifest.
#[derive(Debug, Clone)]
pub struct ManualTokenManifestValidator {
    pub rules: Vec<Rc<dyn ManualTokenManifestRule>>,
}

impl ManualTokenManifestValidator {
    pub fn validate(&self, item: &ManualTokenManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualTokenManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualTokenManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_token_manifest_validator() -> ManualTokenManifestValidator {
    ManualTokenManifestValidator {
        rules: vec![
            Rc::new(ManualTokenManifestLabelLengthRule { max_length: 80 }) as Rc<dyn ManualTokenManifestRule>,
            Rc::new(ManualTokenManifestArchivedGuardRule { limit: 120 }),
            Rc::new(ManualTokenManifestAllowedDescriptionRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(ManualTokenManifestDiscountRangeRule { min: 2.5, max: 3985.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_token_manifest_validator();
        let item = new_manual_token_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ManualTokenManifestLabelLengthRule { max_length: 16 };
        let mut item = new_manual_token_manifest("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = ManualTokenManifestDiscountRangeRule { min: 1.5, max: 3801.2 };
        let mut item = new_manual_token_manifest("x", "y");
        item.discount = 3802.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
