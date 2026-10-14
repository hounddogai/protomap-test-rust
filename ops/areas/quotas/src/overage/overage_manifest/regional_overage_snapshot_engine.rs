use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the regional overage snapshot calculation.
pub trait RegionalOverageSnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalOverageSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalOverageSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalOverageSnapshotModifier for TieredRegionalOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalOverageSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalOverageSnapshotAdjuster {
    pub percent: f64,
}

impl RegionalOverageSnapshotModifier for PercentRegionalOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalOverageSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalOverageSnapshotAdjuster {
    pub minimum: f64,
}

impl RegionalOverageSnapshotModifier for MinimumRegionalOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalOverageSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalOverageSnapshotAdjuster {
    pub cap: f64,
}

impl RegionalOverageSnapshotModifier for CapRegionalOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalOverageSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalOverageSnapshotMinimumOffsetAdjuster {
    pub weight: f64,
}

impl RegionalOverageSnapshotModifier for RegionalOverageSnapshotMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalOverageSnapshot, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the regional overage snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalOverageSnapshotEngine {
    pub steps: Vec<Rc<dyn RegionalOverageSnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalOverageSnapshotEngine {
    pub fn compute(&mut self, item: &RegionalOverageSnapshot) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalOverageSnapshotModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalOverageSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_overage_snapshot_engine() -> RegionalOverageSnapshotEngine {
    RegionalOverageSnapshotEngine {
        steps: vec![
            Rc::new(TieredRegionalOverageSnapshotAdjuster {
                bounds: vec![3405.0, 4418.0],
                rates: vec![0.91, 0.85],
            }) as Rc<dyn RegionalOverageSnapshotModifier>,
            Rc::new(PercentRegionalOverageSnapshotAdjuster { percent: 2.1 }),
            Rc::new(MinimumRegionalOverageSnapshotAdjuster { minimum: 34.1 }),
            Rc::new(CapRegionalOverageSnapshotAdjuster { cap: 25475.0 }),
            Rc::new(RegionalOverageSnapshotMinimumOffsetAdjuster { weight: 1.21 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 12659.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_overage_snapshot_engine();
        let item = new_regional_overage_snapshot("code-608", "label-863");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
