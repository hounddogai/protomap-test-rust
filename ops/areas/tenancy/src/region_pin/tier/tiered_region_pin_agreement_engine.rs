use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered region pin agreement calculation.
pub trait TieredRegionPinAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredRegionPinAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredRegionPinAgreementAdjuster {
    pub step: f64,
}

impl TieredRegionPinAgreementModifier for RoundingTieredRegionPinAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredRegionPinAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredRegionPinAgreementAdjuster {
    pub percent: f64,
}

impl TieredRegionPinAgreementModifier for PercentTieredRegionPinAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredRegionPinAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredRegionPinAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredRegionPinAgreementModifier for FlaggedTieredRegionPinAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredRegionPinAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterTieredRegionPinAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredRegionPinAgreementModifier for ClusterTieredRegionPinAgreementAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &TieredRegionPinAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredRegionPinAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredRegionPinAgreementModifier for TieredTieredRegionPinAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredRegionPinAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinAgreementAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl TieredRegionPinAgreementModifier for TieredRegionPinAgreementAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &TieredRegionPinAgreement, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

/// Runs the tiered region pin agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredRegionPinAgreementEngine {
    pub steps: Vec<Rc<dyn TieredRegionPinAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredRegionPinAgreementEngine {
    pub fn compute(&mut self, item: &TieredRegionPinAgreement) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[TieredRegionPinAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_region_pin_agreement_engine(flags: &flags::Registry) -> TieredRegionPinAgreementEngine {
    TieredRegionPinAgreementEngine {
        steps: vec![
            Rc::new(RoundingTieredRegionPinAgreementAdjuster { step: 0.1 }) as Rc<dyn TieredRegionPinAgreementModifier>,
            Rc::new(PercentTieredRegionPinAgreementAdjuster { percent: -22.5 }),
            Rc::new(FlaggedTieredRegionPinAgreementAdjuster {
                flags: flags.clone(),
                flag: "tiered-region-pin-agreement-default".to_string(),
                percent: 6.5,
            }),
            Rc::new(ClusterTieredRegionPinAgreementAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.22)]),
            }),
            Rc::new(TieredTieredRegionPinAgreementAdjuster {
                bounds: vec![8056.0, 8331.0],
                rates: vec![1.16, 1.01],
            }),
            Rc::new(TieredRegionPinAgreementAttemptsOffsetAdjuster { weight: 1.26 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 87275.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_region_pin_agreement_engine(&flags::new_registry());
        let item = new_tiered_region_pin_agreement("code-69", "title-351");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
