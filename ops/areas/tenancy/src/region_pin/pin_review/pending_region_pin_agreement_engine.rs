use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending region pin agreement calculation.
pub trait PendingRegionPinAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRegionPinAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingRegionPinAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingRegionPinAgreementModifier for FlaggedPendingRegionPinAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingRegionPinAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinAgreementRateOffsetAdjuster {
    pub weight: f64,
}

impl PendingRegionPinAgreementModifier for PendingRegionPinAgreementRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &PendingRegionPinAgreement, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingRegionPinAgreementAdjuster {
    pub step: f64,
}

impl PendingRegionPinAgreementModifier for RoundingPendingRegionPinAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingRegionPinAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingRegionPinAgreementAdjuster {
    pub cap: f64,
}

impl PendingRegionPinAgreementModifier for CapPendingRegionPinAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingRegionPinAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingRegionPinAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingRegionPinAgreementModifier for TieredPendingRegionPinAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingRegionPinAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending region pin agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRegionPinAgreementEngine {
    pub steps: Vec<Rc<dyn PendingRegionPinAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRegionPinAgreementEngine {
    pub fn evaluate(&mut self, item: &PendingRegionPinAgreement) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[PendingRegionPinAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_region_pin_agreement_engine(flags: &flags::Registry) -> PendingRegionPinAgreementEngine {
    PendingRegionPinAgreementEngine {
        steps: vec![
            Rc::new(FlaggedPendingRegionPinAgreementAdjuster {
                flags: flags.clone(),
                flag: "pending-region-pin-agreement-lenient".to_string(),
                percent: 1.4,
            }) as Rc<dyn PendingRegionPinAgreementModifier>,
            Rc::new(PendingRegionPinAgreementRateOffsetAdjuster { weight: 0.42 }),
            Rc::new(RoundingPendingRegionPinAgreementAdjuster { step: 0.25 }),
            Rc::new(CapPendingRegionPinAgreementAdjuster { cap: 21542.0 }),
            Rc::new(TieredPendingRegionPinAgreementAdjuster {
                bounds: vec![4346.0, 7885.0],
                rates: vec![1.19, 1.1],
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 63698.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_region_pin_agreement_engine(&flags::new_registry());
        let item = new_pending_region_pin_agreement("code-507", "title-228");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
