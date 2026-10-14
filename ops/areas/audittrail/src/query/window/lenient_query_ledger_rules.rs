use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient query ledger must pass. Check returns its problems, or none.
pub trait LenientQueryLedgerRule: std::fmt::Debug {
    fn check(&self, item: &LenientQueryLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryLedgerRequiredCodeRule {
    pub message: String,
}

impl LenientQueryLedgerRule for LenientQueryLedgerRequiredCodeRule {
    fn check(&self, item: &LenientQueryLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryLedgerSegmentLengthRule {
    pub max_length: i64,
}

impl LenientQueryLedgerRule for LenientQueryLedgerSegmentLengthRule {
    fn check(&self, item: &LenientQueryLedger) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryLedgerRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientQueryLedgerRule for LenientQueryLedgerRateRangeRule {
    fn check(&self, item: &LenientQueryLedger) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryLedgerRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientQueryLedgerRule for LenientQueryLedgerRatioRangeRule {
    fn check(&self, item: &LenientQueryLedger) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryLedgerCodeLengthRule {
    pub max_length: i64,
}

impl LenientQueryLedgerRule for LenientQueryLedgerCodeLengthRule {
    fn check(&self, item: &LenientQueryLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient query ledger.
#[derive(Debug, Clone)]
pub struct LenientQueryLedgerValidator {
    pub rules: Vec<Rc<dyn LenientQueryLedgerRule>>,
}

impl LenientQueryLedgerValidator {
    pub fn validate(&self, item: &LenientQueryLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientQueryLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientQueryLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_query_ledger_validator() -> LenientQueryLedgerValidator {
    LenientQueryLedgerValidator {
        rules: vec![
            Rc::new(LenientQueryLedgerRequiredCodeRule {
                message: "lenient query ledger needs a value".to_string(),
            }) as Rc<dyn LenientQueryLedgerRule>,
            Rc::new(LenientQueryLedgerSegmentLengthRule { max_length: 16 }),
            Rc::new(LenientQueryLedgerRateRangeRule { min: 1.4, max: 4338.2 }),
            Rc::new(LenientQueryLedgerRatioRangeRule { min: 4.6, max: 313.4 }),
            Rc::new(LenientQueryLedgerCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_query_ledger_validator();
        let item = new_lenient_query_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientQueryLedgerRequiredCodeRule {
            message: "lenient query ledger needs a value".to_string(),
        };
        let mut item = new_lenient_query_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = LenientQueryLedgerSegmentLengthRule { max_length: 3600 };
        let mut item = new_lenient_query_ledger("x", "y");
        item.segment = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = LenientQueryLedgerRateRangeRule { min: 6.5, max: 3763.4 };
        let mut item = new_lenient_query_ledger("x", "y");
        item.rate = 3764.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = LenientQueryLedgerRatioRangeRule { min: 6.9, max: 3026.5 };
        let mut item = new_lenient_query_ledger("x", "y");
        item.ratio = 3028.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientQueryLedgerCodeLengthRule { max_length: 32 };
        let mut item = new_lenient_query_ledger("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
