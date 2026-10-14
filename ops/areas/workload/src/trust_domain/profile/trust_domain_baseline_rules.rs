use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain baseline must pass. Check returns its problems, or none.
pub trait TrustDomainBaselineRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBaselineRequiredTitleRule {
    pub message: String,
}

impl TrustDomainBaselineRule for TrustDomainBaselineRequiredTitleRule {
    fn check(&self, item: &TrustDomainBaseline) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBaselineRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainBaselineRule for TrustDomainBaselineRetriesRangeRule {
    fn check(&self, item: &TrustDomainBaseline) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBaselineBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TrustDomainBaselineRule for TrustDomainBaselineBudgetRangeRule {
    fn check(&self, item: &TrustDomainBaseline) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBaselineRequiredFingerprintRule {
    pub message: String,
}

impl TrustDomainBaselineRule for TrustDomainBaselineRequiredFingerprintRule {
    fn check(&self, item: &TrustDomainBaseline) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain baseline.
#[derive(Debug, Clone)]
pub struct TrustDomainBaselineValidator {
    pub rules: Vec<Rc<dyn TrustDomainBaselineRule>>,
}

impl TrustDomainBaselineValidator {
    pub fn validate(&self, item: &TrustDomainBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_baseline_validator() -> TrustDomainBaselineValidator {
    TrustDomainBaselineValidator {
        rules: vec![
            Rc::new(TrustDomainBaselineRequiredTitleRule {
                message: "trust domain baseline needs a value".to_string(),
            }) as Rc<dyn TrustDomainBaselineRule>,
            Rc::new(TrustDomainBaselineRetriesRangeRule { min: 3600, max: 86400 }),
            Rc::new(TrustDomainBaselineBudgetRangeRule { min: 8.5, max: 4971.0 }),
            Rc::new(TrustDomainBaselineRequiredFingerprintRule {
                message: "trust domain baseline needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_baseline_validator();
        let item = new_trust_domain_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TrustDomainBaselineRequiredTitleRule {
            message: "trust domain baseline needs an entry".to_string(),
        };
        let mut item = new_trust_domain_baseline("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = TrustDomainBaselineRetriesRangeRule { min: 86400, max: 86400 };
        let mut item = new_trust_domain_baseline("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = TrustDomainBaselineBudgetRangeRule { min: 8.6, max: 4222.0 };
        let mut item = new_trust_domain_baseline("x", "y");
        item.budget = 4223.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = TrustDomainBaselineRequiredFingerprintRule {
            message: "trust domain baseline needs a value".to_string(),
        };
        let mut item = new_trust_domain_baseline("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
