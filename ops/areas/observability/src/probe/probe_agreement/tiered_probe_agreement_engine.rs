use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered probe agreement calculation.
pub trait TieredProbeAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredProbeAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredProbeAgreementAdjuster {
    pub step: f64,
}

impl TieredProbeAgreementModifier for RoundingTieredProbeAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredProbeAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredProbeAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredProbeAgreementModifier for FlaggedTieredProbeAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredProbeAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredProbeAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredProbeAgreementModifier for TieredTieredProbeAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredProbeAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredProbeAgreementAdjuster {
    pub minimum: f64,
}

impl TieredProbeAgreementModifier for MinimumTieredProbeAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredProbeAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered probe agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredProbeAgreementEngine {
    pub steps: Vec<Rc<dyn TieredProbeAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredProbeAgreementEngine {
    pub fn compute(&mut self, item: &TieredProbeAgreement) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[TieredProbeAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_probe_agreement_engine(flags: &flags::Registry) -> TieredProbeAgreementEngine {
    TieredProbeAgreementEngine {
        steps: vec![
            Rc::new(RoundingTieredProbeAgreementAdjuster { step: 0.25 }) as Rc<dyn TieredProbeAgreementModifier>,
            Rc::new(FlaggedTieredProbeAgreementAdjuster {
                flags: flags.clone(),
                flag: "tiered-probe-agreement-weighted".to_string(),
                percent: 9.4,
            }),
            Rc::new(TieredTieredProbeAgreementAdjuster {
                bounds: vec![383.0, 5389.0, 7047.0],
                rates: vec![0.85, 1.05, 0.94],
            }),
            Rc::new(MinimumTieredProbeAgreementAdjuster { minimum: 44.0 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 69773.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_probe_agreement_engine(&flags::new_registry());
        let item = new_tiered_probe_agreement("code-594", "name-854");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
