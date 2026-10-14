use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global escalation manifest calculation.
pub trait GlobalEscalationManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalEscalationManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalEscalationManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalEscalationManifestAdjuster for TieredGlobalEscalationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalEscalationManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalEscalationManifestAdjuster {
    pub cap: f64,
}

impl GlobalEscalationManifestAdjuster for CapGlobalEscalationManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalEscalationManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalEscalationManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalEscalationManifestAdjuster for FlaggedGlobalEscalationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalEscalationManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitGlobalEscalationManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalEscalationManifestAdjuster for UnitGlobalEscalationManifestAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &GlobalEscalationManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalEscalationManifestAdjuster {
    pub minimum: f64,
}

impl GlobalEscalationManifestAdjuster for MinimumGlobalEscalationManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalEscalationManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalEscalationManifestAdjuster {
    pub step: f64,
}

impl GlobalEscalationManifestAdjuster for RoundingGlobalEscalationManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalEscalationManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global escalation manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalEscalationManifestEngine {
    pub steps: Vec<Rc<dyn GlobalEscalationManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalEscalationManifestEngine {
    pub fn run(&mut self, item: &GlobalEscalationManifest) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[GlobalEscalationManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_escalation_manifest_engine(flags: &flags::Registry) -> GlobalEscalationManifestEngine {
    GlobalEscalationManifestEngine {
        steps: vec![
            Rc::new(TieredGlobalEscalationManifestAdjuster {
                bounds: vec![3576.0, 6222.0],
                rates: vec![1.01, 1.01],
            }) as Rc<dyn GlobalEscalationManifestAdjuster>,
            Rc::new(CapGlobalEscalationManifestAdjuster { cap: 595.0 }),
            Rc::new(FlaggedGlobalEscalationManifestAdjuster {
                flags: flags.clone(),
                flag: "global-escalation-manifest-weighted".to_string(),
                percent: 7.4,
            }),
            Rc::new(UnitGlobalEscalationManifestAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.0)]),
            }),
            Rc::new(MinimumGlobalEscalationManifestAdjuster { minimum: 44.9 }),
            Rc::new(RoundingGlobalEscalationManifestAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 75705.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_escalation_manifest_engine(&flags::new_registry());
        let item = new_global_escalation_manifest("code-234", "title-87");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
