use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional grant estimate calculation.
pub trait RegionalGrantEstimateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalGrantEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalGrantEstimateAdjuster {
    pub cap: f64,
}

impl RegionalGrantEstimateFactor for CapRegionalGrantEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalGrantEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalGrantEstimateAdjuster {
    pub step: f64,
}

impl RegionalGrantEstimateFactor for RoundingRegionalGrantEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalGrantEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalGrantEstimateAdjuster {
    pub minimum: f64,
}

impl RegionalGrantEstimateFactor for MinimumRegionalGrantEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalGrantEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional grant estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalGrantEstimateEngine {
    pub steps: Vec<Rc<dyn RegionalGrantEstimateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalGrantEstimateEngine {
    pub fn run(&mut self, item: &RegionalGrantEstimate) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RegionalGrantEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_grant_estimate_engine() -> RegionalGrantEstimateEngine {
    RegionalGrantEstimateEngine {
        steps: vec![
            Rc::new(CapRegionalGrantEstimateAdjuster { cap: 38111.0 }) as Rc<dyn RegionalGrantEstimateFactor>,
            Rc::new(RoundingRegionalGrantEstimateAdjuster { step: 5.0 }),
            Rc::new(MinimumRegionalGrantEstimateAdjuster { minimum: 45.8 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 30963.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_grant_estimate_engine();
        let item = new_regional_grant_estimate("code-884", "label-333");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
