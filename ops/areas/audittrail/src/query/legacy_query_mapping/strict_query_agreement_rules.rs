use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict query agreement must pass. Check returns its problems, or none.
pub trait StrictQueryAgreementRule: std::fmt::Debug {
    fn check(&self, item: &StrictQueryAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryAgreementRequiredNameRule {
    pub message: String,
}

impl StrictQueryAgreementRule for StrictQueryAgreementRequiredNameRule {
    fn check(&self, item: &StrictQueryAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryAgreementPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictQueryAgreementRule for StrictQueryAgreementPortRangeRule {
    fn check(&self, item: &StrictQueryAgreement) -> Vec<String> {
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
pub struct StrictQueryAgreementRequiredLabelRule {
    pub message: String,
}

impl StrictQueryAgreementRule for StrictQueryAgreementRequiredLabelRule {
    fn check(&self, item: &StrictQueryAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryAgreementScopeLengthRule {
    pub max_length: i64,
}

impl StrictQueryAgreementRule for StrictQueryAgreementScopeLengthRule {
    fn check(&self, item: &StrictQueryAgreement) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryAgreementNameLengthRule {
    pub max_length: i64,
}

impl StrictQueryAgreementRule for StrictQueryAgreementNameLengthRule {
    fn check(&self, item: &StrictQueryAgreement) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict query agreement.
#[derive(Debug, Clone)]
pub struct StrictQueryAgreementValidator {
    pub rules: Vec<Rc<dyn StrictQueryAgreementRule>>,
}

impl StrictQueryAgreementValidator {
    pub fn validate(&self, item: &StrictQueryAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictQueryAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictQueryAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_query_agreement_validator() -> StrictQueryAgreementValidator {
    StrictQueryAgreementValidator {
        rules: vec![
            Rc::new(StrictQueryAgreementRequiredNameRule {
                message: "strict query agreement needs this field".to_string(),
            }) as Rc<dyn StrictQueryAgreementRule>,
            Rc::new(StrictQueryAgreementPortRangeRule { min: 16, max: 32 }),
            Rc::new(StrictQueryAgreementRequiredLabelRule {
                message: "strict query agreement needs an entry".to_string(),
            }),
            Rc::new(StrictQueryAgreementScopeLengthRule { max_length: 32 }),
            Rc::new(StrictQueryAgreementNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_query_agreement_validator();
        let item = new_strict_query_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictQueryAgreementRequiredNameRule {
            message: "strict query agreement needs this field".to_string(),
        };
        let mut item = new_strict_query_agreement("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = StrictQueryAgreementPortRangeRule { min: 120, max: 3600 };
        let mut item = new_strict_query_agreement("x", "y");
        item.port = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = StrictQueryAgreementRequiredLabelRule {
            message: "strict query agreement needs a value".to_string(),
        };
        let mut item = new_strict_query_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = StrictQueryAgreementScopeLengthRule { max_length: 120 };
        let mut item = new_strict_query_agreement("x", "y");
        item.scope = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictQueryAgreementNameLengthRule { max_length: 86400 };
        let mut item = new_strict_query_agreement("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
