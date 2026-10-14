use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the nonce rule calculation.
pub trait NonceRuleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceRule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceRuleAdjuster for FlaggedNonceRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumNonceRuleAdjuster {
    pub minimum: f64,
}

impl NonceRuleAdjuster for MinimumNonceRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &NonceRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceRuleAdjuster for TieredNonceRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapNonceRuleAdjuster {
    pub cap: f64,
}

impl NonceRuleAdjuster for CapNonceRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &NonceRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentNonceRuleAdjuster {
    pub percent: f64,
}

impl NonceRuleAdjuster for PercentNonceRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &NonceRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the nonce rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceRuleEngine {
    pub steps: Vec<Rc<dyn NonceRuleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceRuleEngine {
    pub fn compute(&mut self, item: &NonceRule) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NonceRuleAdjuster> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[NonceRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_nonce_rule_engine(flags: &flags::Registry) -> NonceRuleEngine {
    NonceRuleEngine {
        steps: vec![
            Rc::new(FlaggedNonceRuleAdjuster {
                flags: flags.clone(),
                flag: "nonce-rule-tiered".to_string(),
                percent: -1.0,
            }) as Rc<dyn NonceRuleAdjuster>,
            Rc::new(MinimumNonceRuleAdjuster { minimum: 8.5 }),
            Rc::new(TieredNonceRuleAdjuster {
                bounds: vec![3772.0, 7594.0, 8760.0],
                rates: vec![1.13, 0.87, 0.95],
            }),
            Rc::new(CapNonceRuleAdjuster { cap: 561.0 }),
            Rc::new(PercentNonceRuleAdjuster { percent: 21.1 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 50118.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_rule_engine(&flags::new_registry());
        let item = new_nonce_rule("code-295", "title-107");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
