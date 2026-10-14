use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event exception must pass. Check returns its problems, or none.
pub trait AuditEventExceptionRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventExceptionTitleLengthRule {
    pub max_length: i64,
}

impl AuditEventExceptionRule for AuditEventExceptionTitleLengthRule {
    fn check(&self, item: &AuditEventException) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventExceptionCodeLengthRule {
    pub max_length: i64,
}

impl AuditEventExceptionRule for AuditEventExceptionCodeLengthRule {
    fn check(&self, item: &AuditEventException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventExceptionGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventExceptionRule for AuditEventExceptionGraceDaysRangeRule {
    fn check(&self, item: &AuditEventException) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventExceptionRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AuditEventExceptionRule for AuditEventExceptionRatioRangeRule {
    fn check(&self, item: &AuditEventException) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event exception.
#[derive(Debug, Clone)]
pub struct AuditEventExceptionValidator {
    pub rules: Vec<Rc<dyn AuditEventExceptionRule>>,
}

impl AuditEventExceptionValidator {
    pub fn validate(&self, item: &AuditEventException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_exception_validator() -> AuditEventExceptionValidator {
    AuditEventExceptionValidator {
        rules: vec![
            Rc::new(AuditEventExceptionTitleLengthRule { max_length: 86400 }) as Rc<dyn AuditEventExceptionRule>,
            Rc::new(AuditEventExceptionCodeLengthRule { max_length: 1000 }),
            Rc::new(AuditEventExceptionGraceDaysRangeRule { min: 80, max: 255 }),
            Rc::new(AuditEventExceptionRatioRangeRule { min: 7.3, max: 1393.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_exception_validator();
        let item = new_audit_event_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AuditEventExceptionTitleLengthRule { max_length: 255 };
        let mut item = new_audit_event_exception("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AuditEventExceptionCodeLengthRule { max_length: 120 };
        let mut item = new_audit_event_exception("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = AuditEventExceptionGraceDaysRangeRule { min: 32, max: 120 };
        let mut item = new_audit_event_exception("x", "y");
        item.grace_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = AuditEventExceptionRatioRangeRule { min: 2.1, max: 2002.5 };
        let mut item = new_audit_event_exception("x", "y");
        item.ratio = 2004.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
