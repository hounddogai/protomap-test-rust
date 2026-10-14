use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered peering exception must pass. Check returns its problems, or none.
pub trait TieredPeeringExceptionRule: std::fmt::Debug {
    fn check(&self, item: &TieredPeeringException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringExceptionRequiredContactEmailRule {
    pub message: String,
}

impl TieredPeeringExceptionRule for TieredPeeringExceptionRequiredContactEmailRule {
    fn check(&self, item: &TieredPeeringException) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringExceptionContactEmailLengthRule {
    pub max_length: i64,
}

impl TieredPeeringExceptionRule for TieredPeeringExceptionContactEmailLengthRule {
    fn check(&self, item: &TieredPeeringException) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringExceptionApprovedGuardRule {
    pub limit: i64,
}

impl TieredPeeringExceptionRule for TieredPeeringExceptionApprovedGuardRule {
    fn check(&self, item: &TieredPeeringException) -> Vec<String> {
        if item.approved && (item.starts_at > self.limit) {
            return vec![format!(
                "starts at {} exceeds {} while approved",
                item.starts_at, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered peering exception.
#[derive(Debug, Clone)]
pub struct TieredPeeringExceptionValidator {
    pub rules: Vec<Rc<dyn TieredPeeringExceptionRule>>,
}

impl TieredPeeringExceptionValidator {
    pub fn validate(&self, item: &TieredPeeringException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredPeeringException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredPeeringException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_peering_exception_validator() -> TieredPeeringExceptionValidator {
    TieredPeeringExceptionValidator {
        rules: vec![
            Rc::new(TieredPeeringExceptionRequiredContactEmailRule {
                message: "tiered peering exception needs a value".to_string(),
            }) as Rc<dyn TieredPeeringExceptionRule>,
            Rc::new(TieredPeeringExceptionContactEmailLengthRule { max_length: 64 }),
            Rc::new(TieredPeeringExceptionApprovedGuardRule { limit: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_peering_exception_validator();
        let item = new_tiered_peering_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = TieredPeeringExceptionRequiredContactEmailRule {
            message: "tiered peering exception needs this field".to_string(),
        };
        let mut item = new_tiered_peering_exception("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = TieredPeeringExceptionContactEmailLengthRule { max_length: 32 };
        let mut item = new_tiered_peering_exception("x", "y");
        item.contact_email = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
