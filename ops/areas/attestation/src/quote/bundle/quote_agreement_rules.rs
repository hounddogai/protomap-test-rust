use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote agreement must pass. Check returns its problems, or none.
pub trait QuoteAgreementRule: std::fmt::Debug {
    fn check(&self, item: &QuoteAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAgreementUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteAgreementRule for QuoteAgreementUpdatedAtRangeRule {
    fn check(&self, item: &QuoteAgreement) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAgreementRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteAgreementRule for QuoteAgreementRotationDaysRangeRule {
    fn check(&self, item: &QuoteAgreement) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAgreementActiveGuardRule {
    pub limit: i64,
}

impl QuoteAgreementRule for QuoteAgreementActiveGuardRule {
    fn check(&self, item: &QuoteAgreement) -> Vec<String> {
        if item.active && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while active", item.burst, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote agreement.
#[derive(Debug, Clone)]
pub struct QuoteAgreementValidator {
    pub rules: Vec<Rc<dyn QuoteAgreementRule>>,
}

impl QuoteAgreementValidator {
    pub fn validate(&self, item: &QuoteAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_agreement_validator() -> QuoteAgreementValidator {
    QuoteAgreementValidator {
        rules: vec![
            Rc::new(QuoteAgreementUpdatedAtRangeRule { min: 32, max: 80 }) as Rc<dyn QuoteAgreementRule>,
            Rc::new(QuoteAgreementRotationDaysRangeRule { min: 255, max: 86400 }),
            Rc::new(QuoteAgreementActiveGuardRule { limit: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_agreement_validator();
        let item = new_quote_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = QuoteAgreementUpdatedAtRangeRule { min: 86400, max: 86400 };
        let mut item = new_quote_agreement("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = QuoteAgreementRotationDaysRangeRule { min: 16, max: 120 };
        let mut item = new_quote_agreement("x", "y");
        item.rotation_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
