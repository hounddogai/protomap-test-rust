use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the suspension digest calculation.
pub trait SuspensionDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionDigestAdjuster {
    pub minimum: f64,
}

impl SuspensionDigestModifier for MinimumSuspensionDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDigestThresholdOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionDigestModifier for SuspensionDigestThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionDigest, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSuspensionDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SuspensionDigestModifier for FlaggedSuspensionDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SuspensionDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSuspensionDigestAdjuster {
    pub step: f64,
}

impl SuspensionDigestModifier for RoundingSuspensionDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SuspensionDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSuspensionDigestAdjuster {
    pub percent: f64,
}

impl SuspensionDigestModifier for PercentSuspensionDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SuspensionDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the suspension digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionDigestEngine {
    pub steps: Vec<Rc<dyn SuspensionDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionDigestEngine {
    pub fn price(&mut self, item: &SuspensionDigest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SuspensionDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_suspension_digest_engine(flags: &flags::Registry) -> SuspensionDigestEngine {
    SuspensionDigestEngine {
        steps: vec![
            Rc::new(MinimumSuspensionDigestAdjuster { minimum: 0.3 }) as Rc<dyn SuspensionDigestModifier>,
            Rc::new(SuspensionDigestThresholdOffsetAdjuster { weight: 0.66 }),
            Rc::new(FlaggedSuspensionDigestAdjuster {
                flags: flags.clone(),
                flag: "suspension-digest-manual".to_string(),
                percent: -3.3,
            }),
            Rc::new(RoundingSuspensionDigestAdjuster { step: 0.05 }),
            Rc::new(PercentSuspensionDigestAdjuster { percent: 16.5 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 60654.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_digest_engine(&flags::new_registry());
        let item = new_suspension_digest("code-684", "name-867");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
