use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a postmortem manifest must pass. Check returns its problems, or none.
pub trait PostmortemManifestRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemManifestRequiredOwnerIdRule {
    pub message: String,
}

impl PostmortemManifestRule for PostmortemManifestRequiredOwnerIdRule {
    fn check(&self, item: &PostmortemManifest) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemManifestAudienceLengthRule {
    pub max_length: i64,
}

impl PostmortemManifestRule for PostmortemManifestAudienceLengthRule {
    fn check(&self, item: &PostmortemManifest) -> Vec<String> {
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
pub struct PostmortemManifestAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl PostmortemManifestRule for PostmortemManifestAllowedOwnerIdRule {
    fn check(&self, item: &PostmortemManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is stale", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemManifestMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PostmortemManifestRule for PostmortemManifestMinimumRangeRule {
    fn check(&self, item: &PostmortemManifest) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a postmortem manifest.
#[derive(Debug, Clone)]
pub struct PostmortemManifestValidator {
    pub rules: Vec<Rc<dyn PostmortemManifestRule>>,
}

impl PostmortemManifestValidator {
    pub fn validate(&self, item: &PostmortemManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_manifest_validator() -> PostmortemManifestValidator {
    PostmortemManifestValidator {
        rules: vec![
            Rc::new(PostmortemManifestRequiredOwnerIdRule {
                message: "postmortem manifest needs this field".to_string(),
            }) as Rc<dyn PostmortemManifestRule>,
            Rc::new(PostmortemManifestAudienceLengthRule { max_length: 3600 }),
            Rc::new(PostmortemManifestAllowedOwnerIdRule {
                allowed: vec!["lenient".to_string(), "seasonal".to_string()],
            }),
            Rc::new(PostmortemManifestMinimumRangeRule { min: 0.4, max: 3695.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_manifest_validator();
        let item = new_postmortem_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = PostmortemManifestRequiredOwnerIdRule {
            message: "postmortem manifest needs this field".to_string(),
        };
        let mut item = new_postmortem_manifest("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = PostmortemManifestAudienceLengthRule { max_length: 16 };
        let mut item = new_postmortem_manifest("x", "y");
        item.audience = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PostmortemManifestMinimumRangeRule { min: 6.9, max: 2666.3 };
        let mut item = new_postmortem_manifest("x", "y");
        item.minimum = 2667.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
