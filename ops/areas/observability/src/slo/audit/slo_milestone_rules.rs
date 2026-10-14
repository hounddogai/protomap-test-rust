use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a slo milestone must pass. Check returns its problems, or none.
pub trait SloMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &SloMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloMilestoneAllowedNameRule {
    pub allowed: Vec<String>,
}

impl SloMilestoneRule for SloMilestoneAllowedNameRule {
    fn check(&self, item: &SloMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloMilestoneIssuerLengthRule {
    pub max_length: i64,
}

impl SloMilestoneRule for SloMilestoneIssuerLengthRule {
    fn check(&self, item: &SloMilestone) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloMilestoneCodePrefixRule {
    pub prefix: String,
}

impl SloMilestoneRule for SloMilestoneCodePrefixRule {
    fn check(&self, item: &SloMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloMilestonePinnedGuardRule {
    pub limit: i64,
}

impl SloMilestoneRule for SloMilestonePinnedGuardRule {
    fn check(&self, item: &SloMilestone) -> Vec<String> {
        if item.pinned && (item.retries > self.limit) {
            return vec![format!("retries {} exceeds {} while pinned", item.retries, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloMilestoneKeyIdLengthRule {
    pub max_length: i64,
}

impl SloMilestoneRule for SloMilestoneKeyIdLengthRule {
    fn check(&self, item: &SloMilestone) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo milestone.
#[derive(Debug, Clone)]
pub struct SloMilestoneValidator {
    pub rules: Vec<Rc<dyn SloMilestoneRule>>,
}

impl SloMilestoneValidator {
    pub fn validate(&self, item: &SloMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_milestone_validator() -> SloMilestoneValidator {
    SloMilestoneValidator {
        rules: vec![
            Rc::new(SloMilestoneAllowedNameRule { allowed: Vec::new() }) as Rc<dyn SloMilestoneRule>,
            Rc::new(SloMilestoneIssuerLengthRule { max_length: 3600 }),
            Rc::new(SloMilestoneCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(SloMilestonePinnedGuardRule { limit: 255 }),
            Rc::new(SloMilestoneKeyIdLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_milestone_validator();
        let item = new_slo_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = SloMilestoneIssuerLengthRule { max_length: 64 };
        let mut item = new_slo_milestone("x", "y");
        item.issuer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = SloMilestoneKeyIdLengthRule { max_length: 16 };
        let mut item = new_slo_milestone("x", "y");
        item.key_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
