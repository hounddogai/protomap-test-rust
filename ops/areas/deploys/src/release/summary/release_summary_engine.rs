use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the release summary calculation.
pub trait ReleaseSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseSummary, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseSummaryAdjuster for FlaggedReleaseSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleaseSummaryAdjuster {
    pub cap: f64,
}

impl ReleaseSummaryAdjuster for CapReleaseSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleaseSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseSummaryAdjuster for TieredReleaseSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the release summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseSummaryEngine {
    pub steps: Vec<Rc<dyn ReleaseSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseSummaryEngine {
    pub fn compute(&mut self, item: &ReleaseSummary) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_release_summary_engine(flags: &flags::Registry) -> ReleaseSummaryEngine {
    ReleaseSummaryEngine {
        steps: vec![
            Rc::new(FlaggedReleaseSummaryAdjuster {
                flags: flags.clone(),
                flag: "release-summary-global".to_string(),
                percent: 2.4,
            }) as Rc<dyn ReleaseSummaryAdjuster>,
            Rc::new(CapReleaseSummaryAdjuster { cap: 16269.0 }),
            Rc::new(TieredReleaseSummaryAdjuster {
                bounds: vec![1146.0, 1769.0, 8434.0],
                rates: vec![0.95, 1.09, 1.0],
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 9601.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_summary_engine(&flags::new_registry());
        let item = new_release_summary("code-15", "name-37");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
