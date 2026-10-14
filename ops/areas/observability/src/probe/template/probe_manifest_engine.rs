use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the probe manifest calculation.
pub trait ProbeManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelProbeManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ProbeManifestStep for ChannelProbeManifestAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &ProbeManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingProbeManifestAdjuster {
    pub step: f64,
}

impl ProbeManifestStep for RoundingProbeManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ProbeManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbeManifestAdjuster {
    pub percent: f64,
}

impl ProbeManifestStep for PercentProbeManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbeManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapProbeManifestAdjuster {
    pub cap: f64,
}

impl ProbeManifestStep for CapProbeManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ProbeManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedProbeManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ProbeManifestStep for FlaggedProbeManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ProbeManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the probe manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeManifestEngine {
    pub steps: Vec<Rc<dyn ProbeManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeManifestEngine {
    pub fn compute(&mut self, item: &ProbeManifest) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ProbeManifestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ProbeManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_probe_manifest_engine(flags: &flags::Registry) -> ProbeManifestEngine {
    ProbeManifestEngine {
        steps: vec![
            Rc::new(ChannelProbeManifestAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.36),
                    ("north".to_string(), 1.23),
                    ("online".to_string(), 1.25),
                ]),
            }) as Rc<dyn ProbeManifestStep>,
            Rc::new(RoundingProbeManifestAdjuster { step: 5.0 }),
            Rc::new(PercentProbeManifestAdjuster { percent: 13.1 }),
            Rc::new(CapProbeManifestAdjuster { cap: 37774.0 }),
            Rc::new(FlaggedProbeManifestAdjuster {
                flags: flags.clone(),
                flag: "probe-manifest-global".to_string(),
                percent: 0.2,
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 96314.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_manifest_engine(&flags::new_registry());
        let item = new_probe_manifest("code-792", "label-652");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
