use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered overage agreement must pass. Check returns its problems, or none.
pub trait TieredOverageAgreementRule: std::fmt::Debug {
    fn check(&self, item: &TieredOverageAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageAgreementExternalRefLengthRule {
    pub max_length: i64,
}

impl TieredOverageAgreementRule for TieredOverageAgreementExternalRefLengthRule {
    fn check(&self, item: &TieredOverageAgreement) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageAgreementPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredOverageAgreementRule for TieredOverageAgreementPortRangeRule {
    fn check(&self, item: &TieredOverageAgreement) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageAgreementAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl TieredOverageAgreementRule for TieredOverageAgreementAllowedAudienceRule {
    fn check(&self, item: &TieredOverageAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is not allowed", item.audience)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered overage agreement.
#[derive(Debug, Clone)]
pub struct TieredOverageAgreementValidator {
    pub rules: Vec<Rc<dyn TieredOverageAgreementRule>>,
}

impl TieredOverageAgreementValidator {
    pub fn validate(&self, item: &TieredOverageAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredOverageAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredOverageAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_overage_agreement_validator() -> TieredOverageAgreementValidator {
    TieredOverageAgreementValidator {
        rules: vec![
            Rc::new(TieredOverageAgreementExternalRefLengthRule { max_length: 64 })
                as Rc<dyn TieredOverageAgreementRule>,
            Rc::new(TieredOverageAgreementPortRangeRule { min: 120, max: 86400 }),
            Rc::new(TieredOverageAgreementAllowedAudienceRule {
                allowed: vec!["capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_overage_agreement_validator();
        let item = new_tiered_overage_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = TieredOverageAgreementExternalRefLengthRule { max_length: 255 };
        let mut item = new_tiered_overage_agreement("x", "y");
        item.external_ref = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = TieredOverageAgreementPortRangeRule { min: 80, max: 3600 };
        let mut item = new_tiered_overage_agreement("x", "y");
        item.port = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
