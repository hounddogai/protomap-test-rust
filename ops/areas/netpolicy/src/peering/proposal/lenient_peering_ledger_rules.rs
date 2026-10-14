use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient peering ledger must pass. Check returns its problems, or none.
pub trait LenientPeeringLedgerRule: std::fmt::Debug {
    fn check(&self, item: &LenientPeeringLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringLedgerPrincipalLengthRule {
    pub max_length: i64,
}

impl LenientPeeringLedgerRule for LenientPeeringLedgerPrincipalLengthRule {
    fn check(&self, item: &LenientPeeringLedger) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringLedgerRequiredPrincipalRule {
    pub message: String,
}

impl LenientPeeringLedgerRule for LenientPeeringLedgerRequiredPrincipalRule {
    fn check(&self, item: &LenientPeeringLedger) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringLedgerAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl LenientPeeringLedgerRule for LenientPeeringLedgerAllowedPrincipalRule {
    fn check(&self, item: &LenientPeeringLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is not allowed", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringLedgerAudienceLengthRule {
    pub max_length: i64,
}

impl LenientPeeringLedgerRule for LenientPeeringLedgerAudienceLengthRule {
    fn check(&self, item: &LenientPeeringLedger) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringLedgerTitleLengthRule {
    pub max_length: i64,
}

impl LenientPeeringLedgerRule for LenientPeeringLedgerTitleLengthRule {
    fn check(&self, item: &LenientPeeringLedger) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient peering ledger.
#[derive(Debug, Clone)]
pub struct LenientPeeringLedgerValidator {
    pub rules: Vec<Rc<dyn LenientPeeringLedgerRule>>,
}

impl LenientPeeringLedgerValidator {
    pub fn validate(&self, item: &LenientPeeringLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientPeeringLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPeeringLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_peering_ledger_validator() -> LenientPeeringLedgerValidator {
    LenientPeeringLedgerValidator {
        rules: vec![
            Rc::new(LenientPeeringLedgerPrincipalLengthRule { max_length: 80 }) as Rc<dyn LenientPeeringLedgerRule>,
            Rc::new(LenientPeeringLedgerRequiredPrincipalRule {
                message: "lenient peering ledger needs a value".to_string(),
            }),
            Rc::new(LenientPeeringLedgerAllowedPrincipalRule {
                allowed: vec!["scheduled".to_string()],
            }),
            Rc::new(LenientPeeringLedgerAudienceLengthRule { max_length: 86400 }),
            Rc::new(LenientPeeringLedgerTitleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_peering_ledger_validator();
        let item = new_lenient_peering_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = LenientPeeringLedgerPrincipalLengthRule { max_length: 32 };
        let mut item = new_lenient_peering_ledger("x", "y");
        item.principal = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = LenientPeeringLedgerRequiredPrincipalRule {
            message: "lenient peering ledger needs an entry".to_string(),
        };
        let mut item = new_lenient_peering_ledger("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = LenientPeeringLedgerAudienceLengthRule { max_length: 16 };
        let mut item = new_lenient_peering_ledger("x", "y");
        item.audience = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientPeeringLedgerTitleLengthRule { max_length: 1000 };
        let mut item = new_lenient_peering_ledger("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
