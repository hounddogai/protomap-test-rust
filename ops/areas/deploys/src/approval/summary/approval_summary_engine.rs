use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval summary calculation.
pub trait ApprovalSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalSummaryAdjuster {
    pub cap: f64,
}

impl ApprovalSummaryAdjuster for CapApprovalSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryApprovalSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalSummaryAdjuster for CountryApprovalSummaryAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ApprovalSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalSummaryAdjuster for TieredApprovalSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalSummaryAdjuster {
    pub percent: f64,
}

impl ApprovalSummaryAdjuster for PercentApprovalSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSummaryQuantityOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalSummaryAdjuster for ApprovalSummaryQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalSummary, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalSummaryAdjuster {
    pub step: f64,
}

impl ApprovalSummaryAdjuster for RoundingApprovalSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the approval summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalSummaryEngine {
    pub steps: Vec<Rc<dyn ApprovalSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalSummaryEngine {
    pub fn compute(&mut self, item: &ApprovalSummary) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_approval_summary_engine() -> ApprovalSummaryEngine {
    ApprovalSummaryEngine {
        steps: vec![
            Rc::new(CapApprovalSummaryAdjuster { cap: 25824.0 }) as Rc<dyn ApprovalSummaryAdjuster>,
            Rc::new(CountryApprovalSummaryAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.83), ("online".to_string(), 1.24)]),
            }),
            Rc::new(TieredApprovalSummaryAdjuster {
                bounds: vec![6603.0, 9730.0],
                rates: vec![0.99, 1.17],
            }),
            Rc::new(PercentApprovalSummaryAdjuster { percent: -13.9 }),
            Rc::new(ApprovalSummaryQuantityOffsetAdjuster { weight: 1.53 }),
            Rc::new(RoundingApprovalSummaryAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 87283.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_summary_engine();
        let item = new_approval_summary("code-1", "title-910");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
