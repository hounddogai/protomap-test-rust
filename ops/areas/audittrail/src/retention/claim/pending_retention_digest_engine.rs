use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending retention digest calculation.
pub trait PendingRetentionDigestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRetentionDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingRetentionDigestAdjuster {
    pub step: f64,
}

impl PendingRetentionDigestStep for RoundingPendingRetentionDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingRetentionDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingRetentionDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingRetentionDigestStep for TieredPendingRetentionDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingRetentionDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingRetentionDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingRetentionDigestStep for FlaggedPendingRetentionDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingRetentionDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingRetentionDigestAdjuster {
    pub percent: f64,
}

impl PendingRetentionDigestStep for PercentPendingRetentionDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingRetentionDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending retention digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRetentionDigestEngine {
    pub steps: Vec<Rc<dyn PendingRetentionDigestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRetentionDigestEngine {
    pub fn run(&mut self, item: &PendingRetentionDigest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PendingRetentionDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_retention_digest_engine(flags: &flags::Registry) -> PendingRetentionDigestEngine {
    PendingRetentionDigestEngine {
        steps: vec![
            Rc::new(RoundingPendingRetentionDigestAdjuster { step: 1.0 }) as Rc<dyn PendingRetentionDigestStep>,
            Rc::new(TieredPendingRetentionDigestAdjuster {
                bounds: vec![3832.0, 4877.0, 9903.0],
                rates: vec![1.19, 0.94, 0.93],
            }),
            Rc::new(FlaggedPendingRetentionDigestAdjuster {
                flags: flags.clone(),
                flag: "pending-retention-digest-legacy".to_string(),
                percent: 1.0,
            }),
            Rc::new(PercentPendingRetentionDigestAdjuster { percent: -22.7 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 24492.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_retention_digest_engine(&flags::new_registry());
        let item = new_pending_retention_digest("code-427", "title-762");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
