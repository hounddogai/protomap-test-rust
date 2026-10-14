use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy allowlist rollup calculation.
pub trait LegacyAllowlistRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyAllowlistRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyAllowlistRollupAdjuster {
    pub percent: f64,
}

impl LegacyAllowlistRollupFactor for PercentLegacyAllowlistRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyAllowlistRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyAllowlistRollupAdjuster {
    pub step: f64,
}

impl LegacyAllowlistRollupFactor for RoundingLegacyAllowlistRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyAllowlistRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyAllowlistRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyAllowlistRollupFactor for TieredLegacyAllowlistRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyAllowlistRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistRollupLimitOffsetAdjuster {
    pub weight: f64,
}

impl LegacyAllowlistRollupFactor for LegacyAllowlistRollupLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyAllowlistRollup, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

/// Runs the legacy allowlist rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyAllowlistRollupEngine {
    pub steps: Vec<Rc<dyn LegacyAllowlistRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyAllowlistRollupEngine {
    pub fn run(&mut self, item: &LegacyAllowlistRollup) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyAllowlistRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyAllowlistRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_allowlist_rollup_engine() -> LegacyAllowlistRollupEngine {
    LegacyAllowlistRollupEngine {
        steps: vec![
            Rc::new(PercentLegacyAllowlistRollupAdjuster { percent: 0.9 }) as Rc<dyn LegacyAllowlistRollupFactor>,
            Rc::new(RoundingLegacyAllowlistRollupAdjuster { step: 1.0 }),
            Rc::new(TieredLegacyAllowlistRollupAdjuster {
                bounds: vec![5381.0, 6402.0, 7523.0, 8952.0],
                rates: vec![0.86, 1.18, 1.17, 0.91],
            }),
            Rc::new(LegacyAllowlistRollupLimitOffsetAdjuster { weight: 1.39 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 3583.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_allowlist_rollup_engine();
        let item = new_legacy_allowlist_rollup("code-683", "label-439");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
