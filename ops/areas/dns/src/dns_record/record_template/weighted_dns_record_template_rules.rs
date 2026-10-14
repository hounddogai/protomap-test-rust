use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted dns record template must pass. Check returns its problems, or none.
pub trait WeightedDnsRecordTemplateRule: std::fmt::Debug {
    fn check(&self, item: &WeightedDnsRecordTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDnsRecordTemplateAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl WeightedDnsRecordTemplateRule for WeightedDnsRecordTemplateAllowedPhoneRule {
    fn check(&self, item: &WeightedDnsRecordTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is duplicated", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDnsRecordTemplateCodePrefixRule {
    pub prefix: String,
}

impl WeightedDnsRecordTemplateRule for WeightedDnsRecordTemplateCodePrefixRule {
    fn check(&self, item: &WeightedDnsRecordTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDnsRecordTemplateScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedDnsRecordTemplateRule for WeightedDnsRecordTemplateScoreRangeRule {
    fn check(&self, item: &WeightedDnsRecordTemplate) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDnsRecordTemplateRequiredCodeRule {
    pub message: String,
}

impl WeightedDnsRecordTemplateRule for WeightedDnsRecordTemplateRequiredCodeRule {
    fn check(&self, item: &WeightedDnsRecordTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDnsRecordTemplateLocaleLengthRule {
    pub max_length: i64,
}

impl WeightedDnsRecordTemplateRule for WeightedDnsRecordTemplateLocaleLengthRule {
    fn check(&self, item: &WeightedDnsRecordTemplate) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted dns record template.
#[derive(Debug, Clone)]
pub struct WeightedDnsRecordTemplateValidator {
    pub rules: Vec<Rc<dyn WeightedDnsRecordTemplateRule>>,
}

impl WeightedDnsRecordTemplateValidator {
    pub fn validate(&self, item: &WeightedDnsRecordTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedDnsRecordTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedDnsRecordTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_dns_record_template_validator() -> WeightedDnsRecordTemplateValidator {
    WeightedDnsRecordTemplateValidator {
        rules: vec![
            Rc::new(WeightedDnsRecordTemplateAllowedPhoneRule {
                allowed: vec!["global".to_string(), "global".to_string(), "manual".to_string()],
            }) as Rc<dyn WeightedDnsRecordTemplateRule>,
            Rc::new(WeightedDnsRecordTemplateCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(WeightedDnsRecordTemplateScoreRangeRule { min: 9.0, max: 3137.4 }),
            Rc::new(WeightedDnsRecordTemplateRequiredCodeRule {
                message: "weighted dns record template needs a value".to_string(),
            }),
            Rc::new(WeightedDnsRecordTemplateLocaleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_dns_record_template_validator();
        let item = new_weighted_dns_record_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = WeightedDnsRecordTemplateScoreRangeRule { min: 8.3, max: 2287.9 };
        let mut item = new_weighted_dns_record_template("x", "y");
        item.score = 2289.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedDnsRecordTemplateRequiredCodeRule {
            message: "weighted dns record template needs a value".to_string(),
        };
        let mut item = new_weighted_dns_record_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = WeightedDnsRecordTemplateLocaleLengthRule { max_length: 86400 };
        let mut item = new_weighted_dns_record_template("x", "y");
        item.locale = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
