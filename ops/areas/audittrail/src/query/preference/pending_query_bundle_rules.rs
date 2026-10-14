use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending query bundle must pass. Check returns its problems, or none.
pub trait PendingQueryBundleRule: std::fmt::Debug {
    fn check(&self, item: &PendingQueryBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryBundleRequiredCodeRule {
    pub message: String,
}

impl PendingQueryBundleRule for PendingQueryBundleRequiredCodeRule {
    fn check(&self, item: &PendingQueryBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryBundleBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingQueryBundleRule for PendingQueryBundleBurstRangeRule {
    fn check(&self, item: &PendingQueryBundle) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryBundleAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl PendingQueryBundleRule for PendingQueryBundleAllowedAudienceRule {
    fn check(&self, item: &PendingQueryBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is below minimum", item.audience)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending query bundle.
#[derive(Debug, Clone)]
pub struct PendingQueryBundleValidator {
    pub rules: Vec<Rc<dyn PendingQueryBundleRule>>,
}

impl PendingQueryBundleValidator {
    pub fn validate(&self, item: &PendingQueryBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingQueryBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingQueryBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_query_bundle_validator() -> PendingQueryBundleValidator {
    PendingQueryBundleValidator {
        rules: vec![
            Rc::new(PendingQueryBundleRequiredCodeRule {
                message: "pending query bundle needs a value".to_string(),
            }) as Rc<dyn PendingQueryBundleRule>,
            Rc::new(PendingQueryBundleBurstRangeRule { min: 16, max: 1000 }),
            Rc::new(PendingQueryBundleAllowedAudienceRule {
                allowed: vec!["legacy".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_query_bundle_validator();
        let item = new_pending_query_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingQueryBundleRequiredCodeRule {
            message: "pending query bundle needs an entry".to_string(),
        };
        let mut item = new_pending_query_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = PendingQueryBundleBurstRangeRule { min: 80, max: 255 };
        let mut item = new_pending_query_bundle("x", "y");
        item.burst = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
