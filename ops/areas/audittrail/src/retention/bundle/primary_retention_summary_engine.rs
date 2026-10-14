use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the primary retention summary calculation.
pub trait PrimaryRetentionSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryRetentionSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryRetentionSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryRetentionSummaryAdjuster for TieredPrimaryRetentionSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryRetentionSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryRetentionSummaryAdjuster {
    pub percent: f64,
}

impl PrimaryRetentionSummaryAdjuster for PercentPrimaryRetentionSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryRetentionSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryRetentionSummaryAdjuster {
    pub cap: f64,
}

impl PrimaryRetentionSummaryAdjuster for CapPrimaryRetentionSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryRetentionSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary retention summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryRetentionSummaryEngine {
    pub steps: Vec<Rc<dyn PrimaryRetentionSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryRetentionSummaryEngine {
    pub fn compute(&mut self, item: &PrimaryRetentionSummary) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryRetentionSummaryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryRetentionSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_retention_summary_engine() -> PrimaryRetentionSummaryEngine {
    PrimaryRetentionSummaryEngine {
        steps: vec![
            Rc::new(TieredPrimaryRetentionSummaryAdjuster {
                bounds: vec![1311.0, 4240.0, 6033.0, 7616.0],
                rates: vec![0.95, 0.82, 1.11, 0.84],
            }) as Rc<dyn PrimaryRetentionSummaryAdjuster>,
            Rc::new(PercentPrimaryRetentionSummaryAdjuster { percent: -24.8 }),
            Rc::new(CapPrimaryRetentionSummaryAdjuster { cap: 22711.0 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 71315.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_retention_summary_engine();
        let item = new_primary_retention_summary("code-666", "label-459");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
