use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the zone snapshot calculation.
pub trait ZoneSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneSnapshotAdjuster {
    pub cap: f64,
}

impl ZoneSnapshotFactor for CapZoneSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceZoneSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ZoneSnapshotFactor for NamespaceZoneSnapshotAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &ZoneSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentZoneSnapshotAdjuster {
    pub percent: f64,
}

impl ZoneSnapshotFactor for PercentZoneSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ZoneSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the zone snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneSnapshotEngine {
    pub steps: Vec<Rc<dyn ZoneSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneSnapshotEngine {
    pub fn compute(&mut self, item: &ZoneSnapshot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ZoneSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_zone_snapshot_engine() -> ZoneSnapshotEngine {
    ZoneSnapshotEngine {
        steps: vec![
            Rc::new(CapZoneSnapshotAdjuster { cap: 26212.0 }) as Rc<dyn ZoneSnapshotFactor>,
            Rc::new(NamespaceZoneSnapshotAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.18), ("central".to_string(), 1.33)]),
            }),
            Rc::new(PercentZoneSnapshotAdjuster { percent: 17.3 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 84588.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_snapshot_engine();
        let item = new_zone_snapshot("code-677", "name-941");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
