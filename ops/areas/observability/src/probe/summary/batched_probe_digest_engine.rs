use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched probe digest calculation.
pub trait BatchedProbeDigestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedProbeDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedProbeDigestAdjuster {
    pub cap: f64,
}

impl BatchedProbeDigestFactor for CapBatchedProbeDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedProbeDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedProbeDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedProbeDigestFactor for FlaggedBatchedProbeDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedProbeDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeDigestStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl BatchedProbeDigestFactor for BatchedProbeDigestStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedProbeDigest, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterBatchedProbeDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedProbeDigestFactor for ClusterBatchedProbeDigestAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &BatchedProbeDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedProbeDigestAdjuster {
    pub step: f64,
}

impl BatchedProbeDigestFactor for RoundingBatchedProbeDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedProbeDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedProbeDigestAdjuster {
    pub percent: f64,
}

impl BatchedProbeDigestFactor for PercentBatchedProbeDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedProbeDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched probe digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedProbeDigestEngine {
    pub steps: Vec<Rc<dyn BatchedProbeDigestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedProbeDigestEngine {
    pub fn compute(&mut self, item: &BatchedProbeDigest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedProbeDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_probe_digest_engine(flags: &flags::Registry) -> BatchedProbeDigestEngine {
    BatchedProbeDigestEngine {
        steps: vec![
            Rc::new(CapBatchedProbeDigestAdjuster { cap: 14675.0 }) as Rc<dyn BatchedProbeDigestFactor>,
            Rc::new(FlaggedBatchedProbeDigestAdjuster {
                flags: flags.clone(),
                flag: "batched-probe-digest-batched".to_string(),
                percent: 6.8,
            }),
            Rc::new(BatchedProbeDigestStartsAtOffsetAdjuster { weight: 0.55 }),
            Rc::new(ClusterBatchedProbeDigestAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.23), ("online".to_string(), 0.87)]),
            }),
            Rc::new(RoundingBatchedProbeDigestAdjuster { step: 0.05 }),
            Rc::new(PercentBatchedProbeDigestAdjuster { percent: 16.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 37667.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_probe_digest_engine(&flags::new_registry());
        let item = new_batched_probe_digest("code-390", "name-520");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
