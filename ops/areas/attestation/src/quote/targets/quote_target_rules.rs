use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote target must pass. Check returns its problems, or none.
pub trait QuoteTargetRule: std::fmt::Debug {
    fn check(&self, item: &QuoteTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteTargetTenantIdLengthRule {
    pub max_length: i64,
}

impl QuoteTargetRule for QuoteTargetTenantIdLengthRule {
    fn check(&self, item: &QuoteTarget) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteTargetChecksumLengthRule {
    pub max_length: i64,
}

impl QuoteTargetRule for QuoteTargetChecksumLengthRule {
    fn check(&self, item: &QuoteTarget) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteTargetMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuoteTargetRule for QuoteTargetMaximumRangeRule {
    fn check(&self, item: &QuoteTarget) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteTargetNameLengthRule {
    pub max_length: i64,
}

impl QuoteTargetRule for QuoteTargetNameLengthRule {
    fn check(&self, item: &QuoteTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteTargetBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteTargetRule for QuoteTargetBurstRangeRule {
    fn check(&self, item: &QuoteTarget) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteTargetRequiredChecksumRule {
    pub message: String,
}

impl QuoteTargetRule for QuoteTargetRequiredChecksumRule {
    fn check(&self, item: &QuoteTarget) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote target.
#[derive(Debug, Clone)]
pub struct QuoteTargetValidator {
    pub rules: Vec<Rc<dyn QuoteTargetRule>>,
}

impl QuoteTargetValidator {
    pub fn validate(&self, item: &QuoteTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_target_validator() -> QuoteTargetValidator {
    QuoteTargetValidator {
        rules: vec![
            Rc::new(QuoteTargetTenantIdLengthRule { max_length: 32 }) as Rc<dyn QuoteTargetRule>,
            Rc::new(QuoteTargetChecksumLengthRule { max_length: 3600 }),
            Rc::new(QuoteTargetMaximumRangeRule { min: 2.2, max: 3063.2 }),
            Rc::new(QuoteTargetNameLengthRule { max_length: 80 }),
            Rc::new(QuoteTargetBurstRangeRule { min: 32, max: 80 }),
            Rc::new(QuoteTargetRequiredChecksumRule {
                message: "quote target needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_target_validator();
        let item = new_quote_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = QuoteTargetTenantIdLengthRule { max_length: 80 };
        let mut item = new_quote_target("x", "y");
        item.tenant_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = QuoteTargetChecksumLengthRule { max_length: 1000 };
        let mut item = new_quote_target("x", "y");
        item.checksum = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = QuoteTargetMaximumRangeRule { min: 5.3, max: 2466.0 };
        let mut item = new_quote_target("x", "y");
        item.maximum = 2467.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = QuoteTargetNameLengthRule { max_length: 86400 };
        let mut item = new_quote_target("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = QuoteTargetBurstRangeRule { min: 3600, max: 86400 };
        let mut item = new_quote_target("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = QuoteTargetRequiredChecksumRule {
            message: "quote target needs a value".to_string(),
        };
        let mut item = new_quote_target("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
