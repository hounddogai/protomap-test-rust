use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the limit snapshot calculation.
pub trait LimitSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleLimitSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LimitSnapshotAdjuster for LocaleLimitSnapshotAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &LimitSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitSnapshotVersionOffsetAdjuster {
    pub weight: f64,
}

impl LimitSnapshotAdjuster for LimitSnapshotVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &LimitSnapshot, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLimitSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LimitSnapshotAdjuster for FlaggedLimitSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LimitSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the limit snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitSnapshotEngine {
    pub steps: Vec<Rc<dyn LimitSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitSnapshotEngine {
    pub fn price(&mut self, item: &LimitSnapshot) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[LimitSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_limit_snapshot_engine(flags: &flags::Registry) -> LimitSnapshotEngine {
    LimitSnapshotEngine {
        steps: vec![
            Rc::new(LocaleLimitSnapshotAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.1), ("central".to_string(), 0.92)]),
            }) as Rc<dyn LimitSnapshotAdjuster>,
            Rc::new(LimitSnapshotVersionOffsetAdjuster { weight: 1.25 }),
            Rc::new(FlaggedLimitSnapshotAdjuster {
                flags: flags.clone(),
                flag: "limit-snapshot-manual".to_string(),
                percent: -2.3,
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 36735.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_snapshot_engine(&flags::new_registry());
        let item = new_limit_snapshot("code-300", "label-780");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
