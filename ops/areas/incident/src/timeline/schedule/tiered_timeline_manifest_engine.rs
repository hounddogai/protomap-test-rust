use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tiered timeline manifest calculation.
pub trait TieredTimelineManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredTimelineManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleTieredTimelineManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredTimelineManifestStep for LocaleTieredTimelineManifestAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &TieredTimelineManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineManifestRatioOffsetAdjuster {
    pub weight: f64,
}

impl TieredTimelineManifestStep for TieredTimelineManifestRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &TieredTimelineManifest, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredTimelineManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredTimelineManifestStep for TieredTieredTimelineManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredTimelineManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredTimelineManifestAdjuster {
    pub cap: f64,
}

impl TieredTimelineManifestStep for CapTieredTimelineManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredTimelineManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the tiered timeline manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredTimelineManifestEngine {
    pub steps: Vec<Rc<dyn TieredTimelineManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredTimelineManifestEngine {
    pub fn run(&mut self, item: &TieredTimelineManifest) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredTimelineManifestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredTimelineManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_timeline_manifest_engine() -> TieredTimelineManifestEngine {
    TieredTimelineManifestEngine {
        steps: vec![
            Rc::new(LocaleTieredTimelineManifestAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.14)]),
            }) as Rc<dyn TieredTimelineManifestStep>,
            Rc::new(TieredTimelineManifestRatioOffsetAdjuster { weight: 0.64 }),
            Rc::new(TieredTieredTimelineManifestAdjuster {
                bounds: vec![2120.0, 8322.0, 9023.0, 9797.0],
                rates: vec![0.81, 1.05, 0.93, 1.17],
            }),
            Rc::new(CapTieredTimelineManifestAdjuster { cap: 35824.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 89431.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_timeline_manifest_engine();
        let item = new_tiered_timeline_manifest("code-594", "label-111");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
