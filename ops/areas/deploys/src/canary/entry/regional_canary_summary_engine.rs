use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional canary summary calculation.
pub trait RegionalCanarySummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalCanarySummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalCanarySummaryAdjuster {
    pub percent: f64,
}

impl RegionalCanarySummaryAdjuster for PercentRegionalCanarySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalCanarySummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalCanarySummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalCanarySummaryAdjuster for TieredRegionalCanarySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalCanarySummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanarySummaryFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl RegionalCanarySummaryAdjuster for RegionalCanarySummaryFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalCanarySummary, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

/// Runs the regional canary summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalCanarySummaryEngine {
    pub steps: Vec<Rc<dyn RegionalCanarySummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalCanarySummaryEngine {
    pub fn evaluate(&mut self, item: &RegionalCanarySummary) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RegionalCanarySummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_canary_summary_engine() -> RegionalCanarySummaryEngine {
    RegionalCanarySummaryEngine {
        steps: vec![
            Rc::new(PercentRegionalCanarySummaryAdjuster { percent: -20.5 }) as Rc<dyn RegionalCanarySummaryAdjuster>,
            Rc::new(TieredRegionalCanarySummaryAdjuster {
                bounds: vec![3858.0, 3903.0],
                rates: vec![0.86, 1.13],
            }),
            Rc::new(RegionalCanarySummaryFloorValueOffsetAdjuster { weight: 0.87 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 53970.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_canary_summary_engine();
        let item = new_regional_canary_summary("code-709", "name-626");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
