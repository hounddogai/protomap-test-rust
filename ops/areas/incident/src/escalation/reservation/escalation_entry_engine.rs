use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the escalation entry calculation.
pub trait EscalationEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationEntryModifier for TieredEscalationEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEscalationEntryAdjuster {
    pub percent: f64,
}

impl EscalationEntryModifier for PercentEscalationEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EscalationEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationEntryAdjuster {
    pub step: f64,
}

impl EscalationEntryModifier for RoundingEscalationEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEscalationEntryAdjuster {
    pub minimum: f64,
}

impl EscalationEntryModifier for MinimumEscalationEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EscalationEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEscalationEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EscalationEntryModifier for FlaggedEscalationEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EscalationEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the escalation entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationEntryEngine {
    pub steps: Vec<Rc<dyn EscalationEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationEntryEngine {
    pub fn price(&mut self, item: &EscalationEntry) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[EscalationEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_escalation_entry_engine(flags: &flags::Registry) -> EscalationEntryEngine {
    EscalationEntryEngine {
        steps: vec![
            Rc::new(TieredEscalationEntryAdjuster {
                bounds: vec![3659.0, 5375.0, 9564.0, 9763.0],
                rates: vec![0.8, 1.04, 0.99, 1.14],
            }) as Rc<dyn EscalationEntryModifier>,
            Rc::new(PercentEscalationEntryAdjuster { percent: 12.6 }),
            Rc::new(RoundingEscalationEntryAdjuster { step: 0.1 }),
            Rc::new(MinimumEscalationEntryAdjuster { minimum: 32.5 }),
            Rc::new(FlaggedEscalationEntryAdjuster {
                flags: flags.clone(),
                flag: "escalation-entry-batched".to_string(),
                percent: 2.6,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 66090.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_entry_engine(&flags::new_registry());
        let item = new_escalation_entry("code-765", "label-306");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
