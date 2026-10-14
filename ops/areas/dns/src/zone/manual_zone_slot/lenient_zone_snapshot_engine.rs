use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient zone snapshot calculation.
pub trait LenientZoneSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientZoneSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientZoneSnapshotAdjuster {
    pub cap: f64,
}

impl LenientZoneSnapshotAdjuster for CapLenientZoneSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientZoneSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientZoneSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientZoneSnapshotAdjuster for FlaggedLenientZoneSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientZoneSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLenientZoneSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientZoneSnapshotAdjuster for LabelLenientZoneSnapshotAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LenientZoneSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneSnapshotEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl LenientZoneSnapshotAdjuster for LenientZoneSnapshotEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &LenientZoneSnapshot, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientZoneSnapshotAdjuster {
    pub step: f64,
}

impl LenientZoneSnapshotAdjuster for RoundingLenientZoneSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientZoneSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientZoneSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientZoneSnapshotAdjuster for TieredLenientZoneSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientZoneSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient zone snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientZoneSnapshotEngine {
    pub steps: Vec<Rc<dyn LenientZoneSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientZoneSnapshotEngine {
    pub fn compute(&mut self, item: &LenientZoneSnapshot) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[LenientZoneSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_zone_snapshot_engine(flags: &flags::Registry) -> LenientZoneSnapshotEngine {
    LenientZoneSnapshotEngine {
        steps: vec![
            Rc::new(CapLenientZoneSnapshotAdjuster { cap: 865.0 }) as Rc<dyn LenientZoneSnapshotAdjuster>,
            Rc::new(FlaggedLenientZoneSnapshotAdjuster {
                flags: flags.clone(),
                flag: "lenient-zone-snapshot-manual".to_string(),
                percent: -9.6,
            }),
            Rc::new(LabelLenientZoneSnapshotAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.12),
                    ("south".to_string(), 1.39),
                    ("east".to_string(), 0.96),
                ]),
            }),
            Rc::new(LenientZoneSnapshotEndsAtOffsetAdjuster { weight: 1.21 }),
            Rc::new(RoundingLenientZoneSnapshotAdjuster { step: 0.25 }),
            Rc::new(TieredLenientZoneSnapshotAdjuster {
                bounds: vec![356.0, 2668.0, 4169.0, 5167.0],
                rates: vec![0.99, 0.81, 0.81, 1.14],
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 87791.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_zone_snapshot_engine(&flags::new_registry());
        let item = new_lenient_zone_snapshot("code-202", "label-413");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
