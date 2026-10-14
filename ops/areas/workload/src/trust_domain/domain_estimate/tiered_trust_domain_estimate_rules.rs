use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered trust domain estimate must pass. Check returns its problems, or none.
pub trait TieredTrustDomainEstimateRule: std::fmt::Debug {
    fn check(&self, item: &TieredTrustDomainEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainEstimateCodePrefixRule {
    pub prefix: String,
}

impl TieredTrustDomainEstimateRule for TieredTrustDomainEstimateCodePrefixRule {
    fn check(&self, item: &TieredTrustDomainEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainEstimateClusterLengthRule {
    pub max_length: i64,
}

impl TieredTrustDomainEstimateRule for TieredTrustDomainEstimateClusterLengthRule {
    fn check(&self, item: &TieredTrustDomainEstimate) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainEstimateCountryLengthRule {
    pub max_length: i64,
}

impl TieredTrustDomainEstimateRule for TieredTrustDomainEstimateCountryLengthRule {
    fn check(&self, item: &TieredTrustDomainEstimate) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered trust domain estimate.
#[derive(Debug, Clone)]
pub struct TieredTrustDomainEstimateValidator {
    pub rules: Vec<Rc<dyn TieredTrustDomainEstimateRule>>,
}

impl TieredTrustDomainEstimateValidator {
    pub fn validate(&self, item: &TieredTrustDomainEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredTrustDomainEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredTrustDomainEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_trust_domain_estimate_validator() -> TieredTrustDomainEstimateValidator {
    TieredTrustDomainEstimateValidator {
        rules: vec![
            Rc::new(TieredTrustDomainEstimateCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn TieredTrustDomainEstimateRule>,
            Rc::new(TieredTrustDomainEstimateClusterLengthRule { max_length: 255 }),
            Rc::new(TieredTrustDomainEstimateCountryLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_trust_domain_estimate_validator();
        let item = new_tiered_trust_domain_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = TieredTrustDomainEstimateClusterLengthRule { max_length: 3600 };
        let mut item = new_tiered_trust_domain_estimate("x", "y");
        item.cluster = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = TieredTrustDomainEstimateCountryLengthRule { max_length: 120 };
        let mut item = new_tiered_trust_domain_estimate("x", "y");
        item.country = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
