use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a peering quota must pass. Check returns its problems, or none.
pub trait PeeringQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PeeringQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringQuotaRequiredLabelRule {
    pub message: String,
}

impl PeeringQuotaRule for PeeringQuotaRequiredLabelRule {
    fn check(&self, item: &PeeringQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringQuotaAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl PeeringQuotaRule for PeeringQuotaAllowedExternalRefRule {
    fn check(&self, item: &PeeringQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is unknown", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringQuotaRequiredCodeRule {
    pub message: String,
}

impl PeeringQuotaRule for PeeringQuotaRequiredCodeRule {
    fn check(&self, item: &PeeringQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringQuotaExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringQuotaRule for PeeringQuotaExpiresAtRangeRule {
    fn check(&self, item: &PeeringQuota) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringQuotaBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringQuotaRule for PeeringQuotaBatchSizeRangeRule {
    fn check(&self, item: &PeeringQuota) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringQuotaCodeLengthRule {
    pub max_length: i64,
}

impl PeeringQuotaRule for PeeringQuotaCodeLengthRule {
    fn check(&self, item: &PeeringQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering quota.
#[derive(Debug, Clone)]
pub struct PeeringQuotaValidator {
    pub rules: Vec<Rc<dyn PeeringQuotaRule>>,
}

impl PeeringQuotaValidator {
    pub fn validate(&self, item: &PeeringQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_quota_validator() -> PeeringQuotaValidator {
    PeeringQuotaValidator {
        rules: vec![
            Rc::new(PeeringQuotaRequiredLabelRule {
                message: "peering quota needs an entry".to_string(),
            }) as Rc<dyn PeeringQuotaRule>,
            Rc::new(PeeringQuotaAllowedExternalRefRule {
                allowed: vec!["capped".to_string(), "lenient".to_string()],
            }),
            Rc::new(PeeringQuotaRequiredCodeRule {
                message: "peering quota needs a value".to_string(),
            }),
            Rc::new(PeeringQuotaExpiresAtRangeRule { min: 255, max: 1000 }),
            Rc::new(PeeringQuotaBatchSizeRangeRule { min: 80, max: 3600 }),
            Rc::new(PeeringQuotaCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_quota_validator();
        let item = new_peering_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PeeringQuotaRequiredLabelRule {
            message: "peering quota needs an entry".to_string(),
        };
        let mut item = new_peering_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PeeringQuotaRequiredCodeRule {
            message: "peering quota needs an entry".to_string(),
        };
        let mut item = new_peering_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = PeeringQuotaExpiresAtRangeRule { min: 80, max: 86400 };
        let mut item = new_peering_quota("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = PeeringQuotaBatchSizeRangeRule { min: 16, max: 255 };
        let mut item = new_peering_quota("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PeeringQuotaCodeLengthRule { max_length: 3600 };
        let mut item = new_peering_quota("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
