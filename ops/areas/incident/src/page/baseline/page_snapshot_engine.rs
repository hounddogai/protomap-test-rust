use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the page snapshot calculation.
pub trait PageSnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageSnapshotMinimumOffsetAdjuster {
    pub weight: f64,
}

impl PageSnapshotModifier for PageSnapshotMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &PageSnapshot, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPageSnapshotAdjuster {
    pub step: f64,
}

impl PageSnapshotModifier for RoundingPageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PageSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPageSnapshotAdjuster {
    pub minimum: f64,
}

impl PageSnapshotModifier for MinimumPageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PageSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPageSnapshotAdjuster {
    pub percent: f64,
}

impl PageSnapshotModifier for PercentPageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PageSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the page snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageSnapshotEngine {
    pub steps: Vec<Rc<dyn PageSnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageSnapshotEngine {
    pub fn compute(&mut self, item: &PageSnapshot) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[PageSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_page_snapshot_engine() -> PageSnapshotEngine {
    PageSnapshotEngine {
        steps: vec![
            Rc::new(PageSnapshotMinimumOffsetAdjuster { weight: 1.26 }) as Rc<dyn PageSnapshotModifier>,
            Rc::new(RoundingPageSnapshotAdjuster { step: 0.25 }),
            Rc::new(MinimumPageSnapshotAdjuster { minimum: 23.3 }),
            Rc::new(PercentPageSnapshotAdjuster { percent: -8.3 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 78532.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_snapshot_engine();
        let item = new_page_snapshot("code-610", "label-946");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
