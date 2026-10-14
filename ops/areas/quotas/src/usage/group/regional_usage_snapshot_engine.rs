use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional usage snapshot calculation.
pub trait RegionalUsageSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalUsageSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeRegionalUsageSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalUsageSnapshotAdjuster for ScopeRegionalUsageSnapshotAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &RegionalUsageSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalUsageSnapshotAdjuster {
    pub cap: f64,
}

impl RegionalUsageSnapshotAdjuster for CapRegionalUsageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalUsageSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageSnapshotThresholdOffsetAdjuster {
    pub weight: f64,
}

impl RegionalUsageSnapshotAdjuster for RegionalUsageSnapshotThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalUsageSnapshot, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalUsageSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalUsageSnapshotAdjuster for TieredRegionalUsageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalUsageSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalUsageSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalUsageSnapshotAdjuster for FlaggedRegionalUsageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalUsageSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional usage snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalUsageSnapshotEngine {
    pub steps: Vec<Rc<dyn RegionalUsageSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalUsageSnapshotEngine {
    pub fn evaluate(&mut self, item: &RegionalUsageSnapshot) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionalUsageSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_usage_snapshot_engine(flags: &flags::Registry) -> RegionalUsageSnapshotEngine {
    RegionalUsageSnapshotEngine {
        steps: vec![
            Rc::new(ScopeRegionalUsageSnapshotAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.73),
                    ("retail".to_string(), 1.34),
                    ("west".to_string(), 1.03),
                ]),
            }) as Rc<dyn RegionalUsageSnapshotAdjuster>,
            Rc::new(CapRegionalUsageSnapshotAdjuster { cap: 28388.0 }),
            Rc::new(RegionalUsageSnapshotThresholdOffsetAdjuster { weight: 1.73 }),
            Rc::new(TieredRegionalUsageSnapshotAdjuster {
                bounds: vec![6772.0, 8278.0, 8944.0, 9888.0],
                rates: vec![1.01, 0.92, 0.87, 1.03],
            }),
            Rc::new(FlaggedRegionalUsageSnapshotAdjuster {
                flags: flags.clone(),
                flag: "regional-usage-snapshot-rolling".to_string(),
                percent: -3.5,
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 64856.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_usage_snapshot_engine(&flags::new_registry());
        let item = new_regional_usage_snapshot("code-678", "name-181");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
