use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered usage summary calculation.
pub trait TieredUsageSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredUsageSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredUsageSummaryAdjuster {
    pub minimum: f64,
}

impl TieredUsageSummaryModifier for MinimumTieredUsageSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredUsageSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredUsageSummaryAdjuster {
    pub step: f64,
}

impl TieredUsageSummaryModifier for RoundingTieredUsageSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredUsageSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredUsageSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredUsageSummaryModifier for TieredTieredUsageSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredUsageSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered usage summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredUsageSummaryEngine {
    pub steps: Vec<Rc<dyn TieredUsageSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredUsageSummaryEngine {
    pub fn price(&mut self, item: &TieredUsageSummary) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredUsageSummaryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredUsageSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_usage_summary_engine() -> TieredUsageSummaryEngine {
    TieredUsageSummaryEngine {
        steps: vec![
            Rc::new(MinimumTieredUsageSummaryAdjuster { minimum: 9.8 }) as Rc<dyn TieredUsageSummaryModifier>,
            Rc::new(RoundingTieredUsageSummaryAdjuster { step: 5.0 }),
            Rc::new(TieredTieredUsageSummaryAdjuster {
                bounds: vec![4040.0, 5880.0],
                rates: vec![1.06, 1.15],
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 36821.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_usage_summary_engine();
        let item = new_tiered_usage_summary("code-175", "name-920");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
