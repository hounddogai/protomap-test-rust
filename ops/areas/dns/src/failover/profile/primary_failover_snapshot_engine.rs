use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary failover snapshot calculation.
pub trait PrimaryFailoverSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryFailoverSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryFailoverSnapshotAdjuster {
    pub cap: f64,
}

impl PrimaryFailoverSnapshotFactor for CapPrimaryFailoverSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryFailoverSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryFailoverSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryFailoverSnapshotFactor for FlaggedPrimaryFailoverSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryFailoverSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPrimaryFailoverSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryFailoverSnapshotFactor for SegmentPrimaryFailoverSnapshotAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &PrimaryFailoverSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverSnapshotGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryFailoverSnapshotFactor for PrimaryFailoverSnapshotGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryFailoverSnapshot, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the primary failover snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryFailoverSnapshotEngine {
    pub steps: Vec<Rc<dyn PrimaryFailoverSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryFailoverSnapshotEngine {
    pub fn compute(&mut self, item: &PrimaryFailoverSnapshot) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryFailoverSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_failover_snapshot_engine(flags: &flags::Registry) -> PrimaryFailoverSnapshotEngine {
    PrimaryFailoverSnapshotEngine {
        steps: vec![
            Rc::new(CapPrimaryFailoverSnapshotAdjuster { cap: 4564.0 }) as Rc<dyn PrimaryFailoverSnapshotFactor>,
            Rc::new(FlaggedPrimaryFailoverSnapshotAdjuster {
                flags: flags.clone(),
                flag: "primary-failover-snapshot-capped".to_string(),
                percent: 3.8,
            }),
            Rc::new(SegmentPrimaryFailoverSnapshotAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.74),
                    ("online".to_string(), 0.71),
                    ("retail".to_string(), 1.24),
                ]),
            }),
            Rc::new(PrimaryFailoverSnapshotGraceDaysOffsetAdjuster { weight: 1.36 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 60279.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_failover_snapshot_engine(&flags::new_registry());
        let item = new_primary_failover_snapshot("code-673", "name-550");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
