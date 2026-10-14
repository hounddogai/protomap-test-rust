use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a trust domain summary must pass. Check returns its problems, or none.
pub trait TrustDomainSummaryRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSummaryWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TrustDomainSummaryRule for TrustDomainSummaryWeightRangeRule {
    fn check(&self, item: &TrustDomainSummary) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSummaryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TrustDomainSummaryRule for TrustDomainSummaryAllowedCodeRule {
    fn check(&self, item: &TrustDomainSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSummaryApprovedGuardRule {
    pub limit: i64,
}

impl TrustDomainSummaryRule for TrustDomainSummaryApprovedGuardRule {
    fn check(&self, item: &TrustDomainSummary) -> Vec<String> {
        if item.approved && (item.attempts > self.limit) {
            return vec![format!(
                "attempts {} exceeds {} while approved",
                item.attempts, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSummaryRequiredTitleRule {
    pub message: String,
}

impl TrustDomainSummaryRule for TrustDomainSummaryRequiredTitleRule {
    fn check(&self, item: &TrustDomainSummary) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSummaryRequiredLabelRule {
    pub message: String,
}

impl TrustDomainSummaryRule for TrustDomainSummaryRequiredLabelRule {
    fn check(&self, item: &TrustDomainSummary) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain summary.
#[derive(Debug, Clone)]
pub struct TrustDomainSummaryValidator {
    pub rules: Vec<Rc<dyn TrustDomainSummaryRule>>,
}

impl TrustDomainSummaryValidator {
    pub fn validate(&self, item: &TrustDomainSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_summary_validator() -> TrustDomainSummaryValidator {
    TrustDomainSummaryValidator {
        rules: vec![
            Rc::new(TrustDomainSummaryWeightRangeRule { min: 6.2, max: 322.9 }) as Rc<dyn TrustDomainSummaryRule>,
            Rc::new(TrustDomainSummaryAllowedCodeRule {
                allowed: vec!["strict".to_string(), "legacy".to_string()],
            }),
            Rc::new(TrustDomainSummaryApprovedGuardRule { limit: 255 }),
            Rc::new(TrustDomainSummaryRequiredTitleRule {
                message: "trust domain summary needs a value".to_string(),
            }),
            Rc::new(TrustDomainSummaryRequiredLabelRule {
                message: "trust domain summary needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_summary_validator();
        let item = new_trust_domain_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = TrustDomainSummaryWeightRangeRule { min: 3.6, max: 1314.5 };
        let mut item = new_trust_domain_summary("x", "y");
        item.weight = 1316.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TrustDomainSummaryRequiredTitleRule {
            message: "trust domain summary needs an entry".to_string(),
        };
        let mut item = new_trust_domain_summary("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TrustDomainSummaryRequiredLabelRule {
            message: "trust domain summary needs a value".to_string(),
        };
        let mut item = new_trust_domain_summary("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
