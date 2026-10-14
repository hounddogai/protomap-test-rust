use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive nonce rule calculation.
pub trait AdaptiveNonceRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveNonceRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveNonceRuleAdjuster {
    pub step: f64,
}

impl AdaptiveNonceRuleModifier for RoundingAdaptiveNonceRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveNonceRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveNonceRuleAdjuster {
    pub cap: f64,
}

impl AdaptiveNonceRuleModifier for CapAdaptiveNonceRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveNonceRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveNonceRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveNonceRuleModifier for TieredAdaptiveNonceRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveNonceRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceRuleMinimumOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveNonceRuleModifier for AdaptiveNonceRuleMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveNonceRule, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveNonceRuleAdjuster {
    pub percent: f64,
}

impl AdaptiveNonceRuleModifier for PercentAdaptiveNonceRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveNonceRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelAdaptiveNonceRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveNonceRuleModifier for ChannelAdaptiveNonceRuleAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &AdaptiveNonceRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the adaptive nonce rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveNonceRuleEngine {
    pub steps: Vec<Rc<dyn AdaptiveNonceRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveNonceRuleEngine {
    pub fn run(&mut self, item: &AdaptiveNonceRule) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveNonceRuleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveNonceRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_nonce_rule_engine() -> AdaptiveNonceRuleEngine {
    AdaptiveNonceRuleEngine {
        steps: vec![
            Rc::new(RoundingAdaptiveNonceRuleAdjuster { step: 0.25 }) as Rc<dyn AdaptiveNonceRuleModifier>,
            Rc::new(CapAdaptiveNonceRuleAdjuster { cap: 28561.0 }),
            Rc::new(TieredAdaptiveNonceRuleAdjuster {
                bounds: vec![30.0, 1719.0, 7528.0],
                rates: vec![1.05, 0.98, 1.11],
            }),
            Rc::new(AdaptiveNonceRuleMinimumOffsetAdjuster { weight: 1.54 }),
            Rc::new(PercentAdaptiveNonceRuleAdjuster { percent: -12.5 }),
            Rc::new(ChannelAdaptiveNonceRuleAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.12),
                    ("online".to_string(), 1.36),
                    ("retail".to_string(), 1.01),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 76663.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_nonce_rule_engine();
        let item = new_adaptive_nonce_rule("code-465", "title-604");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
