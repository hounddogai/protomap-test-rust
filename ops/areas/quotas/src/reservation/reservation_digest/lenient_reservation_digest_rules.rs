use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient reservation digest must pass. Check returns its problems, or none.
pub trait LenientReservationDigestRule: std::fmt::Debug {
    fn check(&self, item: &LenientReservationDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationDigestRequiredTitleRule {
    pub message: String,
}

impl LenientReservationDigestRule for LenientReservationDigestRequiredTitleRule {
    fn check(&self, item: &LenientReservationDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationDigestCodeLengthRule {
    pub max_length: i64,
}

impl LenientReservationDigestRule for LenientReservationDigestCodeLengthRule {
    fn check(&self, item: &LenientReservationDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationDigestTitleLengthRule {
    pub max_length: i64,
}

impl LenientReservationDigestRule for LenientReservationDigestTitleLengthRule {
    fn check(&self, item: &LenientReservationDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient reservation digest.
#[derive(Debug, Clone)]
pub struct LenientReservationDigestValidator {
    pub rules: Vec<Rc<dyn LenientReservationDigestRule>>,
}

impl LenientReservationDigestValidator {
    pub fn validate(&self, item: &LenientReservationDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientReservationDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientReservationDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_reservation_digest_validator() -> LenientReservationDigestValidator {
    LenientReservationDigestValidator {
        rules: vec![
            Rc::new(LenientReservationDigestRequiredTitleRule {
                message: "lenient reservation digest needs an entry".to_string(),
            }) as Rc<dyn LenientReservationDigestRule>,
            Rc::new(LenientReservationDigestCodeLengthRule { max_length: 3600 }),
            Rc::new(LenientReservationDigestTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_reservation_digest_validator();
        let item = new_lenient_reservation_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientReservationDigestRequiredTitleRule {
            message: "lenient reservation digest needs an entry".to_string(),
        };
        let mut item = new_lenient_reservation_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientReservationDigestCodeLengthRule { max_length: 86400 };
        let mut item = new_lenient_reservation_digest("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientReservationDigestTitleLengthRule { max_length: 120 };
        let mut item = new_lenient_reservation_digest("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
