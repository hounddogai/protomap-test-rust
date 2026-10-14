use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient trust domain score must pass. Check returns its problems, or none.
pub trait LenientTrustDomainScoreRule: std::fmt::Debug {
    fn check(&self, item: &LenientTrustDomainScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTrustDomainScoreRequiredLabelRule {
    pub message: String,
}

impl LenientTrustDomainScoreRule for LenientTrustDomainScoreRequiredLabelRule {
    fn check(&self, item: &LenientTrustDomainScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTrustDomainScoreScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientTrustDomainScoreRule for LenientTrustDomainScoreScoreRangeRule {
    fn check(&self, item: &LenientTrustDomainScore) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTrustDomainScoreCodePrefixRule {
    pub prefix: String,
}

impl LenientTrustDomainScoreRule for LenientTrustDomainScoreCodePrefixRule {
    fn check(&self, item: &LenientTrustDomainScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient trust domain score.
#[derive(Debug, Clone)]
pub struct LenientTrustDomainScoreValidator {
    pub rules: Vec<Rc<dyn LenientTrustDomainScoreRule>>,
}

impl LenientTrustDomainScoreValidator {
    pub fn validate(&self, item: &LenientTrustDomainScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientTrustDomainScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTrustDomainScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_trust_domain_score_validator() -> LenientTrustDomainScoreValidator {
    LenientTrustDomainScoreValidator {
        rules: vec![
            Rc::new(LenientTrustDomainScoreRequiredLabelRule {
                message: "lenient trust domain score needs an entry".to_string(),
            }) as Rc<dyn LenientTrustDomainScoreRule>,
            Rc::new(LenientTrustDomainScoreScoreRangeRule { min: 3.2, max: 1237.2 }),
            Rc::new(LenientTrustDomainScoreCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
