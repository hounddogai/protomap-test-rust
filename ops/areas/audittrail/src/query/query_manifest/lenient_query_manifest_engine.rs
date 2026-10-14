use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient query manifest calculation.
pub trait LenientQueryManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientQueryManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryManifestPriorityOffsetAdjuster {
    pub weight: f64,
}

impl LenientQueryManifestStep for LenientQueryManifestPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &LenientQueryManifest, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientQueryManifestAdjuster {
    pub cap: f64,
}

impl LenientQueryManifestStep for CapLenientQueryManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientQueryManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientQueryManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientQueryManifestStep for FlaggedLenientQueryManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientQueryManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientQueryManifestAdjuster {
    pub percent: f64,
}

impl LenientQueryManifestStep for PercentLenientQueryManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientQueryManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientQueryManifestAdjuster {
    pub step: f64,
}

impl LenientQueryManifestStep for RoundingLenientQueryManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientQueryManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentLenientQueryManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientQueryManifestStep for SegmentLenientQueryManifestAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &LenientQueryManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient query manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientQueryManifestEngine {
    pub steps: Vec<Rc<dyn LenientQueryManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientQueryManifestEngine {
    pub fn compute(&mut self, item: &LenientQueryManifest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientQueryManifestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientQueryManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_query_manifest_engine(flags: &flags::Registry) -> LenientQueryManifestEngine {
    LenientQueryManifestEngine {
        steps: vec![
            Rc::new(LenientQueryManifestPriorityOffsetAdjuster { weight: 0.94 }) as Rc<dyn LenientQueryManifestStep>,
            Rc::new(CapLenientQueryManifestAdjuster { cap: 36955.0 }),
            Rc::new(FlaggedLenientQueryManifestAdjuster {
                flags: flags.clone(),
                flag: "lenient-query-manifest-seasonal".to_string(),
                percent: -0.0,
            }),
            Rc::new(PercentLenientQueryManifestAdjuster { percent: -9.7 }),
            Rc::new(RoundingLenientQueryManifestAdjuster { step: 1.0 }),
            Rc::new(SegmentLenientQueryManifestAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.77),
                    ("south".to_string(), 0.73),
                    ("east".to_string(), 1.31),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 10846.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_query_manifest_engine(&flags::new_registry());
        let item = new_lenient_query_manifest("code-344", "name-466");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
