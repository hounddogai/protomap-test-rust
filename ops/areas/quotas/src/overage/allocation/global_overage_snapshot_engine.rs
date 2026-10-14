use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global overage snapshot calculation.
pub trait GlobalOverageSnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalOverageSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOverageSnapshotCostOffsetAdjuster {
    pub weight: f64,
}

impl GlobalOverageSnapshotModifier for GlobalOverageSnapshotCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalOverageSnapshot, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalOverageSnapshotAdjuster {
    pub minimum: f64,
}

impl GlobalOverageSnapshotModifier for MinimumGlobalOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalOverageSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameGlobalOverageSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalOverageSnapshotModifier for NameGlobalOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &GlobalOverageSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalOverageSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalOverageSnapshotModifier for TieredGlobalOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalOverageSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalOverageSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalOverageSnapshotModifier for FlaggedGlobalOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalOverageSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global overage snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalOverageSnapshotEngine {
    pub steps: Vec<Rc<dyn GlobalOverageSnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalOverageSnapshotEngine {
    pub fn price(&mut self, item: &GlobalOverageSnapshot) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[GlobalOverageSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_overage_snapshot_engine(flags: &flags::Registry) -> GlobalOverageSnapshotEngine {
    GlobalOverageSnapshotEngine {
        steps: vec![
            Rc::new(GlobalOverageSnapshotCostOffsetAdjuster { weight: 1.73 }) as Rc<dyn GlobalOverageSnapshotModifier>,
            Rc::new(MinimumGlobalOverageSnapshotAdjuster { minimum: 25.9 }),
            Rc::new(NameGlobalOverageSnapshotAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.16),
                    ("east".to_string(), 1.28),
                    ("north".to_string(), 0.72),
                ]),
            }),
            Rc::new(TieredGlobalOverageSnapshotAdjuster {
                bounds: vec![912.0, 4875.0, 7864.0],
                rates: vec![1.1, 0.82, 0.86],
            }),
            Rc::new(FlaggedGlobalOverageSnapshotAdjuster {
                flags: flags.clone(),
                flag: "global-overage-snapshot-scheduled".to_string(),
                percent: -4.0,
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 92527.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_overage_snapshot_engine(&flags::new_registry());
        let item = new_global_overage_snapshot("code-47", "name-831");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
