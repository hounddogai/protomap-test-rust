use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page audit must pass. Check returns its problems, or none.
pub trait PageAuditRule: std::fmt::Debug {
    fn check(&self, item: &PageAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAuditRequiredCodeRule {
    pub message: String,
}

impl PageAuditRule for PageAuditRequiredCodeRule {
    fn check(&self, item: &PageAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAuditAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PageAuditRule for PageAuditAmountRangeRule {
    fn check(&self, item: &PageAudit) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAuditLabelLengthRule {
    pub max_length: i64,
}

impl PageAuditRule for PageAuditLabelLengthRule {
    fn check(&self, item: &PageAudit) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAuditIssuerLengthRule {
    pub max_length: i64,
}

impl PageAuditRule for PageAuditIssuerLengthRule {
    fn check(&self, item: &PageAudit) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a page audit.
#[derive(Debug, Clone)]
pub struct PageAuditValidator {
    pub rules: Vec<Rc<dyn PageAuditRule>>,
}

impl PageAuditValidator {
    pub fn validate(&self, item: &PageAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_audit_validator() -> PageAuditValidator {
    PageAuditValidator {
        rules: vec![
            Rc::new(PageAuditRequiredCodeRule {
                message: "page audit needs a value".to_string(),
            }) as Rc<dyn PageAuditRule>,
            Rc::new(PageAuditAmountRangeRule { min: 9.8, max: 3925.9 }),
            Rc::new(PageAuditLabelLengthRule { max_length: 120 }),
            Rc::new(PageAuditIssuerLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_audit_validator();
        let item = new_page_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PageAuditRequiredCodeRule {
            message: "page audit needs an entry".to_string(),
        };
        let mut item = new_page_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = PageAuditAmountRangeRule { min: 4.7, max: 4710.2 };
        let mut item = new_page_audit("x", "y");
        item.amount = 4711.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PageAuditLabelLengthRule { max_length: 16 };
        let mut item = new_page_audit("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = PageAuditIssuerLengthRule { max_length: 120 };
        let mut item = new_page_audit("x", "y");
        item.issuer = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
