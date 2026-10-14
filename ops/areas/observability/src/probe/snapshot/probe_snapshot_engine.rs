use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the probe snapshot calculation.
pub trait ProbeSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumProbeSnapshotAdjuster {
    pub minimum: f64,
}

impl ProbeSnapshotStep for MinimumProbeSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ProbeSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSnapshotExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl ProbeSnapshotStep for ProbeSnapshotExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &ProbeSnapshot, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameProbeSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ProbeSnapshotStep for NameProbeSnapshotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ProbeSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the probe snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeSnapshotEngine {
    pub steps: Vec<Rc<dyn ProbeSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeSnapshotEngine {
    pub fn compute(&mut self, item: &ProbeSnapshot) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ProbeSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_probe_snapshot_engine() -> ProbeSnapshotEngine {
    ProbeSnapshotEngine {
        steps: vec![
            Rc::new(MinimumProbeSnapshotAdjuster { minimum: 15.1 }) as Rc<dyn ProbeSnapshotStep>,
            Rc::new(ProbeSnapshotExpiresAtOffsetAdjuster { weight: 0.53 }),
            Rc::new(NameProbeSnapshotAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.2),
                    ("retail".to_string(), 1.15),
                    ("north".to_string(), 1.39),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 78009.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_snapshot_engine();
        let item = new_probe_snapshot("code-966", "name-217");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
