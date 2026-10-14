use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the adaptive query tier calculation.
pub trait AdaptiveQueryTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveQueryTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveQueryTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveQueryTierModifier for TieredAdaptiveQueryTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveQueryTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveQueryTierAdjuster {
    pub minimum: f64,
}

impl AdaptiveQueryTierModifier for MinimumAdaptiveQueryTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveQueryTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveQueryTierAdjuster {
    pub cap: f64,
}

impl AdaptiveQueryTierModifier for CapAdaptiveQueryTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveQueryTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveQueryTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveQueryTierModifier for FlaggedAdaptiveQueryTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveQueryTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveQueryTierAdjuster {
    pub percent: f64,
}

impl AdaptiveQueryTierModifier for PercentAdaptiveQueryTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveQueryTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive query tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveQueryTierEngine {
    pub steps: Vec<Rc<dyn AdaptiveQueryTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveQueryTierEngine {
    pub fn price(&mut self, item: &AdaptiveQueryTier) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveQueryTierModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveQueryTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_query_tier_engine(flags: &flags::Registry) -> AdaptiveQueryTierEngine {
    AdaptiveQueryTierEngine {
        steps: vec![
            Rc::new(TieredAdaptiveQueryTierAdjuster {
                bounds: vec![2478.0, 3544.0, 7486.0],
                rates: vec![1.17, 0.99, 0.86],
            }) as Rc<dyn AdaptiveQueryTierModifier>,
            Rc::new(MinimumAdaptiveQueryTierAdjuster { minimum: 8.1 }),
            Rc::new(CapAdaptiveQueryTierAdjuster { cap: 24077.0 }),
            Rc::new(FlaggedAdaptiveQueryTierAdjuster {
                flags: flags.clone(),
                flag: "adaptive-query-tier-lenient".to_string(),
                percent: -6.0,
            }),
            Rc::new(PercentAdaptiveQueryTierAdjuster { percent: 1.3 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 82246.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_query_tier_engine(&flags::new_registry());
        let item = new_adaptive_query_tier("code-310", "label-533");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
