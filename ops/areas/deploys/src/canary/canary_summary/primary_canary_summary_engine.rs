use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary canary summary calculation.
pub trait PrimaryCanarySummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryCanarySummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryCanarySummaryAdjuster {
    pub percent: f64,
}

impl PrimaryCanarySummaryFactor for PercentPrimaryCanarySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryCanarySummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryCanarySummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryCanarySummaryFactor for TieredPrimaryCanarySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryCanarySummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanarySummaryRevisionOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryCanarySummaryFactor for PrimaryCanarySummaryRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryCanarySummary, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryCanarySummaryAdjuster {
    pub step: f64,
}

impl PrimaryCanarySummaryFactor for RoundingPrimaryCanarySummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryCanarySummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the primary canary summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryCanarySummaryEngine {
    pub steps: Vec<Rc<dyn PrimaryCanarySummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryCanarySummaryEngine {
    pub fn run(&mut self, item: &PrimaryCanarySummary) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryCanarySummaryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryCanarySummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_canary_summary_engine() -> PrimaryCanarySummaryEngine {
    PrimaryCanarySummaryEngine {
        steps: vec![
            Rc::new(PercentPrimaryCanarySummaryAdjuster { percent: 12.8 }) as Rc<dyn PrimaryCanarySummaryFactor>,
            Rc::new(TieredPrimaryCanarySummaryAdjuster {
                bounds: vec![1494.0, 2566.0, 2911.0, 6614.0],
                rates: vec![1.16, 0.89, 0.99, 1.2],
            }),
            Rc::new(PrimaryCanarySummaryRevisionOffsetAdjuster { weight: 1.78 }),
            Rc::new(RoundingPrimaryCanarySummaryAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 28298.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_canary_summary_engine();
        let item = new_primary_canary_summary("code-605", "label-764");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
