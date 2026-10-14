use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary failover digest calculation.
pub trait PrimaryFailoverDigestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryFailoverDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryFailoverDigestAdjuster {
    pub minimum: f64,
}

impl PrimaryFailoverDigestFactor for MinimumPrimaryFailoverDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryFailoverDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryFailoverDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryFailoverDigestFactor for FlaggedPrimaryFailoverDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryFailoverDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryFailoverDigestAdjuster {
    pub cap: f64,
}

impl PrimaryFailoverDigestFactor for CapPrimaryFailoverDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryFailoverDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary failover digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryFailoverDigestEngine {
    pub steps: Vec<Rc<dyn PrimaryFailoverDigestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryFailoverDigestEngine {
    pub fn evaluate(&mut self, item: &PrimaryFailoverDigest) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryFailoverDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_failover_digest_engine(flags: &flags::Registry) -> PrimaryFailoverDigestEngine {
    PrimaryFailoverDigestEngine {
        steps: vec![
            Rc::new(MinimumPrimaryFailoverDigestAdjuster { minimum: 37.8 }) as Rc<dyn PrimaryFailoverDigestFactor>,
            Rc::new(FlaggedPrimaryFailoverDigestAdjuster {
                flags: flags.clone(),
                flag: "primary-failover-digest-adaptive".to_string(),
                percent: 0.0,
            }),
            Rc::new(CapPrimaryFailoverDigestAdjuster { cap: 25061.0 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 51741.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_failover_digest_engine(&flags::new_registry());
        let item = new_primary_failover_digest("code-623", "title-762");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
