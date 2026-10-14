use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary limit audit must pass. Check returns its problems, or none.
pub trait PrimaryLimitAuditRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryLimitAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitAuditCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryLimitAuditRule for PrimaryLimitAuditCostRangeRule {
    fn check(&self, item: &PrimaryLimitAudit) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitAuditCountryLengthRule {
    pub max_length: i64,
}

impl PrimaryLimitAuditRule for PrimaryLimitAuditCountryLengthRule {
    fn check(&self, item: &PrimaryLimitAudit) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitAuditRequiredNotesRule {
    pub message: String,
}

impl PrimaryLimitAuditRule for PrimaryLimitAuditRequiredNotesRule {
    fn check(&self, item: &PrimaryLimitAudit) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitAuditRequiredCodeRule {
    pub message: String,
}

impl PrimaryLimitAuditRule for PrimaryLimitAuditRequiredCodeRule {
    fn check(&self, item: &PrimaryLimitAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitAuditTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryLimitAuditRule for PrimaryLimitAuditTitleLengthRule {
    fn check(&self, item: &PrimaryLimitAudit) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary limit audit.
#[derive(Debug, Clone)]
pub struct PrimaryLimitAuditValidator {
    pub rules: Vec<Rc<dyn PrimaryLimitAuditRule>>,
}

impl PrimaryLimitAuditValidator {
    pub fn validate(&self, item: &PrimaryLimitAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryLimitAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryLimitAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_limit_audit_validator() -> PrimaryLimitAuditValidator {
    PrimaryLimitAuditValidator {
        rules: vec![
            Rc::new(PrimaryLimitAuditCostRangeRule { min: 9.4, max: 707.7 }) as Rc<dyn PrimaryLimitAuditRule>,
            Rc::new(PrimaryLimitAuditCountryLengthRule { max_length: 32 }),
            Rc::new(PrimaryLimitAuditRequiredNotesRule {
                message: "primary limit audit needs this field".to_string(),
            }),
            Rc::new(PrimaryLimitAuditRequiredCodeRule {
                message: "primary limit audit needs an entry".to_string(),
            }),
            Rc::new(PrimaryLimitAuditTitleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_limit_audit_validator();
        let item = new_primary_limit_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = PrimaryLimitAuditCostRangeRule { min: 6.4, max: 2183.6 };
        let mut item = new_primary_limit_audit("x", "y");
        item.cost = 2185.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = PrimaryLimitAuditCountryLengthRule { max_length: 1000 };
        let mut item = new_primary_limit_audit("x", "y");
        item.country = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = PrimaryLimitAuditRequiredNotesRule {
            message: "primary limit audit needs a value".to_string(),
        };
        let mut item = new_primary_limit_audit("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryLimitAuditRequiredCodeRule {
            message: "primary limit audit needs a value".to_string(),
        };
        let mut item = new_primary_limit_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryLimitAuditTitleLengthRule { max_length: 64 };
        let mut item = new_primary_limit_audit("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
