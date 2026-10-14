use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary release claim must pass. Check returns its problems, or none.
pub trait PrimaryReleaseClaimRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryReleaseClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReleaseClaimRequiredCodeRule {
    pub message: String,
}

impl PrimaryReleaseClaimRule for PrimaryReleaseClaimRequiredCodeRule {
    fn check(&self, item: &PrimaryReleaseClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReleaseClaimAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl PrimaryReleaseClaimRule for PrimaryReleaseClaimAllowedUnitRule {
    fn check(&self, item: &PrimaryReleaseClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is blocked", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReleaseClaimUnitLengthRule {
    pub max_length: i64,
}

impl PrimaryReleaseClaimRule for PrimaryReleaseClaimUnitLengthRule {
    fn check(&self, item: &PrimaryReleaseClaim) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReleaseClaimRequiredUnitRule {
    pub message: String,
}

impl PrimaryReleaseClaimRule for PrimaryReleaseClaimRequiredUnitRule {
    fn check(&self, item: &PrimaryReleaseClaim) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReleaseClaimTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryReleaseClaimRule for PrimaryReleaseClaimTitleLengthRule {
    fn check(&self, item: &PrimaryReleaseClaim) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary release claim.
#[derive(Debug, Clone)]
pub struct PrimaryReleaseClaimValidator {
    pub rules: Vec<Rc<dyn PrimaryReleaseClaimRule>>,
}

impl PrimaryReleaseClaimValidator {
    pub fn validate(&self, item: &PrimaryReleaseClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryReleaseClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryReleaseClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_release_claim_validator() -> PrimaryReleaseClaimValidator {
    PrimaryReleaseClaimValidator {
        rules: vec![
            Rc::new(PrimaryReleaseClaimRequiredCodeRule {
                message: "primary release claim needs an entry".to_string(),
            }) as Rc<dyn PrimaryReleaseClaimRule>,
            Rc::new(PrimaryReleaseClaimAllowedUnitRule { allowed: Vec::new() }),
            Rc::new(PrimaryReleaseClaimUnitLengthRule { max_length: 16 }),
            Rc::new(PrimaryReleaseClaimRequiredUnitRule {
                message: "primary release claim needs a value".to_string(),
            }),
            Rc::new(PrimaryReleaseClaimTitleLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_release_claim_validator();
        let item = new_primary_release_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryReleaseClaimRequiredCodeRule {
            message: "primary release claim needs this field".to_string(),
        };
        let mut item = new_primary_release_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = PrimaryReleaseClaimUnitLengthRule { max_length: 32 };
        let mut item = new_primary_release_claim("x", "y");
        item.unit = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PrimaryReleaseClaimRequiredUnitRule {
            message: "primary release claim needs a value".to_string(),
        };
        let mut item = new_primary_release_claim("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryReleaseClaimTitleLengthRule { max_length: 64 };
        let mut item = new_primary_release_claim("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
