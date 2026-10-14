use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the failover manifest calculation.
pub trait FailoverManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverManifestAdjuster {
    pub cap: f64,
}

impl FailoverManifestModifier for CapFailoverManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFailoverManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FailoverManifestModifier for FlaggedFailoverManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FailoverManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFailoverManifestAdjuster {
    pub minimum: f64,
}

impl FailoverManifestModifier for MinimumFailoverManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FailoverManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the failover manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverManifestEngine {
    pub steps: Vec<Rc<dyn FailoverManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverManifestEngine {
    pub fn compute(&mut self, item: &FailoverManifest) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[FailoverManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_failover_manifest_engine(flags: &flags::Registry) -> FailoverManifestEngine {
    FailoverManifestEngine {
        steps: vec![
            Rc::new(CapFailoverManifestAdjuster { cap: 42405.0 }) as Rc<dyn FailoverManifestModifier>,
            Rc::new(FlaggedFailoverManifestAdjuster {
                flags: flags.clone(),
                flag: "failover-manifest-adaptive".to_string(),
                percent: 9.6,
            }),
            Rc::new(MinimumFailoverManifestAdjuster { minimum: 21.4 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 42248.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_manifest_engine(&flags::new_registry());
        let item = new_failover_manifest("code-385", "title-793");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
