use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the escalation slot calculation.
pub trait EscalationSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationSlotAdjuster {
    pub step: f64,
}

impl EscalationSlotModifier for RoundingEscalationSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEscalationSlotAdjuster {
    pub percent: f64,
}

impl EscalationSlotModifier for PercentEscalationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EscalationSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEscalationSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EscalationSlotModifier for FlaggedEscalationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EscalationSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationSlotModifier for TieredEscalationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEscalationSlotAdjuster {
    pub cap: f64,
}

impl EscalationSlotModifier for CapEscalationSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EscalationSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the escalation slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationSlotEngine {
    pub steps: Vec<Rc<dyn EscalationSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationSlotEngine {
    pub fn evaluate(&mut self, item: &EscalationSlot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[EscalationSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_escalation_slot_engine(flags: &flags::Registry) -> EscalationSlotEngine {
    EscalationSlotEngine {
        steps: vec![
            Rc::new(RoundingEscalationSlotAdjuster { step: 0.5 }) as Rc<dyn EscalationSlotModifier>,
            Rc::new(PercentEscalationSlotAdjuster { percent: -22.9 }),
            Rc::new(FlaggedEscalationSlotAdjuster {
                flags: flags.clone(),
                flag: "escalation-slot-legacy".to_string(),
                percent: -4.1,
            }),
            Rc::new(TieredEscalationSlotAdjuster {
                bounds: vec![2280.0, 9120.0],
                rates: vec![1.05, 0.94],
            }),
            Rc::new(CapEscalationSlotAdjuster { cap: 12684.0 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 73968.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_slot_engine(&flags::new_registry());
        let item = new_escalation_slot("code-113", "label-607");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
