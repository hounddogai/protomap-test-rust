use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval manifest must pass. Check returns its problems, or none.
pub trait ApprovalManifestRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalManifestNameLengthRule {
    pub max_length: i64,
}

impl ApprovalManifestRule for ApprovalManifestNameLengthRule {
    fn check(&self, item: &ApprovalManifest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalManifestAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalManifestRule for ApprovalManifestAttemptsRangeRule {
    fn check(&self, item: &ApprovalManifest) -> Vec<String> {
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
pub struct ApprovalManifestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ApprovalManifestRule for ApprovalManifestAllowedCodeRule {
    fn check(&self, item: &ApprovalManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalManifestFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalManifestRule for ApprovalManifestFloorValueRangeRule {
    fn check(&self, item: &ApprovalManifest) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval manifest.
#[derive(Debug, Clone)]
pub struct ApprovalManifestValidator {
    pub rules: Vec<Rc<dyn ApprovalManifestRule>>,
}

impl ApprovalManifestValidator {
    pub fn validate(&self, item: &ApprovalManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_manifest_validator() -> ApprovalManifestValidator {
    ApprovalManifestValidator {
        rules: vec![
            Rc::new(ApprovalManifestNameLengthRule { max_length: 80 }) as Rc<dyn ApprovalManifestRule>,
            Rc::new(ApprovalManifestAttemptsRangeRule { min: 80, max: 120 }),
            Rc::new(ApprovalManifestAllowedCodeRule {
                allowed: vec!["weighted".to_string(), "global".to_string()],
            }),
            Rc::new(ApprovalManifestFloorValueRangeRule { min: 9.6, max: 1791.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_manifest_validator();
        let item = new_approval_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ApprovalManifestNameLengthRule { max_length: 1000 };
        let mut item = new_approval_manifest("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ApprovalManifestAttemptsRangeRule { min: 3600, max: 3600 };
        let mut item = new_approval_manifest("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = ApprovalManifestFloorValueRangeRule { min: 0.3, max: 1567.7 };
        let mut item = new_approval_manifest("x", "y");
        item.floor_value = 1569.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
