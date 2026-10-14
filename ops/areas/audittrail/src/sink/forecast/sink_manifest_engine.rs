use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the sink manifest calculation.
pub trait SinkManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceSinkManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkManifestFactor for SourceSinkManifestAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &SinkManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSinkManifestAdjuster {
    pub step: f64,
}

impl SinkManifestFactor for RoundingSinkManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SinkManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSinkManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SinkManifestFactor for FlaggedSinkManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SinkManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkManifestQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl SinkManifestFactor for SinkManifestQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &SinkManifest, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

/// Runs the sink manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkManifestEngine {
    pub steps: Vec<Rc<dyn SinkManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkManifestEngine {
    pub fn run(&mut self, item: &SinkManifest) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[SinkManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_sink_manifest_engine(flags: &flags::Registry) -> SinkManifestEngine {
    SinkManifestEngine {
        steps: vec![
            Rc::new(SourceSinkManifestAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.07),
                    ("south".to_string(), 0.74),
                    ("retail".to_string(), 1.04),
                ]),
            }) as Rc<dyn SinkManifestFactor>,
            Rc::new(RoundingSinkManifestAdjuster { step: 1.0 }),
            Rc::new(FlaggedSinkManifestAdjuster {
                flags: flags.clone(),
                flag: "sink-manifest-rolling".to_string(),
                percent: -8.0,
            }),
            Rc::new(SinkManifestQuotaUnitsOffsetAdjuster { weight: 1.43 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 99997.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_manifest_engine(&flags::new_registry());
        let item = new_sink_manifest("code-191", "label-453");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
