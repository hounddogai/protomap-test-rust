use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered page snapshot calculation.
pub trait TieredPageSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredPageSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredPageSnapshotAdjuster {
    pub minimum: f64,
}

impl TieredPageSnapshotFactor for MinimumTieredPageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredPageSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredPageSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredPageSnapshotFactor for TieredTieredPageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredPageSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredPageSnapshotAdjuster {
    pub percent: f64,
}

impl TieredPageSnapshotFactor for PercentTieredPageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredPageSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleTieredPageSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredPageSnapshotFactor for TitleTieredPageSnapshotAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &TieredPageSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageSnapshotSequenceOffsetAdjuster {
    pub weight: f64,
}

impl TieredPageSnapshotFactor for TieredPageSnapshotSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &TieredPageSnapshot, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredPageSnapshotAdjuster {
    pub cap: f64,
}

impl TieredPageSnapshotFactor for CapTieredPageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredPageSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the tiered page snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredPageSnapshotEngine {
    pub steps: Vec<Rc<dyn TieredPageSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredPageSnapshotEngine {
    pub fn price(&mut self, item: &TieredPageSnapshot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TieredPageSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_page_snapshot_engine() -> TieredPageSnapshotEngine {
    TieredPageSnapshotEngine {
        steps: vec![
            Rc::new(MinimumTieredPageSnapshotAdjuster { minimum: 19.4 }) as Rc<dyn TieredPageSnapshotFactor>,
            Rc::new(TieredTieredPageSnapshotAdjuster {
                bounds: vec![864.0, 1174.0, 3122.0, 9003.0],
                rates: vec![0.92, 1.04, 0.87, 0.88],
            }),
            Rc::new(PercentTieredPageSnapshotAdjuster { percent: 11.4 }),
            Rc::new(TitleTieredPageSnapshotAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.81), ("central".to_string(), 0.84)]),
            }),
            Rc::new(TieredPageSnapshotSequenceOffsetAdjuster { weight: 0.85 }),
            Rc::new(CapTieredPageSnapshotAdjuster { cap: 37582.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 31600.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_page_snapshot_engine();
        let item = new_tiered_page_snapshot("code-400", "title-393");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
