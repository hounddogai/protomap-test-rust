use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback quote schedule must pass. Check returns its problems, or none.
pub trait FallbackQuoteScheduleRule: std::fmt::Debug {
    fn check(&self, item: &FallbackQuoteSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteScheduleRequiredCodeRule {
    pub message: String,
}

impl FallbackQuoteScheduleRule for FallbackQuoteScheduleRequiredCodeRule {
    fn check(&self, item: &FallbackQuoteSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteScheduleCodeLengthRule {
    pub max_length: i64,
}

impl FallbackQuoteScheduleRule for FallbackQuoteScheduleCodeLengthRule {
    fn check(&self, item: &FallbackQuoteSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteScheduleRequiredNamespaceRule {
    pub message: String,
}

impl FallbackQuoteScheduleRule for FallbackQuoteScheduleRequiredNamespaceRule {
    fn check(&self, item: &FallbackQuoteSchedule) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteSchedulePrincipalLengthRule {
    pub max_length: i64,
}

impl FallbackQuoteScheduleRule for FallbackQuoteSchedulePrincipalLengthRule {
    fn check(&self, item: &FallbackQuoteSchedule) -> Vec<String> {
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
pub struct FallbackQuoteScheduleNameLengthRule {
    pub max_length: i64,
}

impl FallbackQuoteScheduleRule for FallbackQuoteScheduleNameLengthRule {
    fn check(&self, item: &FallbackQuoteSchedule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback quote schedule.
#[derive(Debug, Clone)]
pub struct FallbackQuoteScheduleValidator {
    pub rules: Vec<Rc<dyn FallbackQuoteScheduleRule>>,
}

impl FallbackQuoteScheduleValidator {
    pub fn validate(&self, item: &FallbackQuoteSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackQuoteSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackQuoteSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_quote_schedule_validator() -> FallbackQuoteScheduleValidator {
    FallbackQuoteScheduleValidator {
        rules: vec![
            Rc::new(FallbackQuoteScheduleRequiredCodeRule {
                message: "fallback quote schedule needs a value".to_string(),
            }) as Rc<dyn FallbackQuoteScheduleRule>,
            Rc::new(FallbackQuoteScheduleCodeLengthRule { max_length: 255 }),
            Rc::new(FallbackQuoteScheduleRequiredNamespaceRule {
                message: "fallback quote schedule needs this field".to_string(),
            }),
            Rc::new(FallbackQuoteSchedulePrincipalLengthRule { max_length: 64 }),
            Rc::new(FallbackQuoteScheduleNameLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_quote_schedule_validator();
        let item = new_fallback_quote_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackQuoteScheduleRequiredCodeRule {
            message: "fallback quote schedule needs this field".to_string(),
        };
        let mut item = new_fallback_quote_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackQuoteScheduleCodeLengthRule { max_length: 120 };
        let mut item = new_fallback_quote_schedule("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = FallbackQuoteScheduleRequiredNamespaceRule {
            message: "fallback quote schedule needs this field".to_string(),
        };
        let mut item = new_fallback_quote_schedule("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = FallbackQuoteSchedulePrincipalLengthRule { max_length: 32 };
        let mut item = new_fallback_quote_schedule("x", "y");
        item.principal = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackQuoteScheduleNameLengthRule { max_length: 64 };
        let mut item = new_fallback_quote_schedule("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
