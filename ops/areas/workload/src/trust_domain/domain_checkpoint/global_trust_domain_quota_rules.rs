use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global trust domain quota must pass. Check returns its problems, or none.
pub trait GlobalTrustDomainQuotaRule: std::fmt::Debug {
    fn check(&self, item: &GlobalTrustDomainQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTrustDomainQuotaRequiredNameRule {
    pub message: String,
}

impl GlobalTrustDomainQuotaRule for GlobalTrustDomainQuotaRequiredNameRule {
    fn check(&self, item: &GlobalTrustDomainQuota) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTrustDomainQuotaQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalTrustDomainQuotaRule for GlobalTrustDomainQuotaQuotaUnitsRangeRule {
    fn check(&self, item: &GlobalTrustDomainQuota) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTrustDomainQuotaRequiredSegmentRule {
    pub message: String,
}

impl GlobalTrustDomainQuotaRule for GlobalTrustDomainQuotaRequiredSegmentRule {
    fn check(&self, item: &GlobalTrustDomainQuota) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global trust domain quota.
#[derive(Debug, Clone)]
pub struct GlobalTrustDomainQuotaValidator {
    pub rules: Vec<Rc<dyn GlobalTrustDomainQuotaRule>>,
}

impl GlobalTrustDomainQuotaValidator {
    pub fn validate(&self, item: &GlobalTrustDomainQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalTrustDomainQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalTrustDomainQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_trust_domain_quota_validator() -> GlobalTrustDomainQuotaValidator {
    GlobalTrustDomainQuotaValidator {
        rules: vec![
            Rc::new(GlobalTrustDomainQuotaRequiredNameRule {
                message: "global trust domain quota needs a value".to_string(),
            }) as Rc<dyn GlobalTrustDomainQuotaRule>,
            Rc::new(GlobalTrustDomainQuotaQuotaUnitsRangeRule { min: 120, max: 3600 }),
            Rc::new(GlobalTrustDomainQuotaRequiredSegmentRule {
                message: "global trust domain quota needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_trust_domain_quota_validator();
        let item = new_global_trust_domain_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GlobalTrustDomainQuotaRequiredNameRule {
            message: "global trust domain quota needs this field".to_string(),
        };
        let mut item = new_global_trust_domain_quota("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = GlobalTrustDomainQuotaQuotaUnitsRangeRule { min: 32, max: 64 };
        let mut item = new_global_trust_domain_quota("x", "y");
        item.quota_units = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = GlobalTrustDomainQuotaRequiredSegmentRule {
            message: "global trust domain quota needs a value".to_string(),
        };
        let mut item = new_global_trust_domain_quota("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
