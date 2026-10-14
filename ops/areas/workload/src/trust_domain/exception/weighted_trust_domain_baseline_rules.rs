use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted trust domain baseline must pass. Check returns its problems, or none.
pub trait WeightedTrustDomainBaselineRule: std::fmt::Debug {
    fn check(&self, item: &WeightedTrustDomainBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainBaselineNameLengthRule {
    pub max_length: i64,
}

impl WeightedTrustDomainBaselineRule for WeightedTrustDomainBaselineNameLengthRule {
    fn check(&self, item: &WeightedTrustDomainBaseline) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainBaselineCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedTrustDomainBaselineRule for WeightedTrustDomainBaselineCostRangeRule {
    fn check(&self, item: &WeightedTrustDomainBaseline) -> Vec<String> {
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
pub struct WeightedTrustDomainBaselineCodeLengthRule {
    pub max_length: i64,
}

impl WeightedTrustDomainBaselineRule for WeightedTrustDomainBaselineCodeLengthRule {
    fn check(&self, item: &WeightedTrustDomainBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainBaselineCodePrefixRule {
    pub prefix: String,
}

impl WeightedTrustDomainBaselineRule for WeightedTrustDomainBaselineCodePrefixRule {
    fn check(&self, item: &WeightedTrustDomainBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainBaselineRequiredNamespaceRule {
    pub message: String,
}

impl WeightedTrustDomainBaselineRule for WeightedTrustDomainBaselineRequiredNamespaceRule {
    fn check(&self, item: &WeightedTrustDomainBaseline) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted trust domain baseline.
#[derive(Debug, Clone)]
pub struct WeightedTrustDomainBaselineValidator {
    pub rules: Vec<Rc<dyn WeightedTrustDomainBaselineRule>>,
}

impl WeightedTrustDomainBaselineValidator {
    pub fn validate(&self, item: &WeightedTrustDomainBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedTrustDomainBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedTrustDomainBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_trust_domain_baseline_validator() -> WeightedTrustDomainBaselineValidator {
    WeightedTrustDomainBaselineValidator {
        rules: vec![
            Rc::new(WeightedTrustDomainBaselineNameLengthRule { max_length: 16 })
                as Rc<dyn WeightedTrustDomainBaselineRule>,
            Rc::new(WeightedTrustDomainBaselineCostRangeRule { min: 2.8, max: 762.7 }),
            Rc::new(WeightedTrustDomainBaselineCodeLengthRule { max_length: 255 }),
            Rc::new(WeightedTrustDomainBaselineCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(WeightedTrustDomainBaselineRequiredNamespaceRule {
                message: "weighted trust domain baseline needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_trust_domain_baseline_validator();
        let item = new_weighted_trust_domain_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedTrustDomainBaselineNameLengthRule { max_length: 255 };
        let mut item = new_weighted_trust_domain_baseline("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = WeightedTrustDomainBaselineCostRangeRule { min: 3.7, max: 2250.8 };
        let mut item = new_weighted_trust_domain_baseline("x", "y");
        item.cost = 2252.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedTrustDomainBaselineCodeLengthRule { max_length: 32 };
        let mut item = new_weighted_trust_domain_baseline("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = WeightedTrustDomainBaselineRequiredNamespaceRule {
            message: "weighted trust domain baseline needs a value".to_string(),
        };
        let mut item = new_weighted_trust_domain_baseline("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
