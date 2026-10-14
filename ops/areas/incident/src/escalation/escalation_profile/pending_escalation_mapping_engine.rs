use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending escalation mapping calculation.
pub trait PendingEscalationMappingFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingEscalationMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingEscalationMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingEscalationMappingFactor for TieredPendingEscalationMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingEscalationMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingEscalationMappingAdjuster {
    pub step: f64,
}

impl PendingEscalationMappingFactor for RoundingPendingEscalationMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingEscalationMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingEscalationMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingEscalationMappingFactor for FlaggedPendingEscalationMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingEscalationMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending escalation mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingEscalationMappingEngine {
    pub steps: Vec<Rc<dyn PendingEscalationMappingFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingEscalationMappingEngine {
    pub fn evaluate(&mut self, item: &PendingEscalationMapping) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PendingEscalationMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_escalation_mapping_engine(flags: &flags::Registry) -> PendingEscalationMappingEngine {
    PendingEscalationMappingEngine {
        steps: vec![
            Rc::new(TieredPendingEscalationMappingAdjuster {
                bounds: vec![8294.0, 8696.0],
                rates: vec![0.89, 0.97],
            }) as Rc<dyn PendingEscalationMappingFactor>,
            Rc::new(RoundingPendingEscalationMappingAdjuster { step: 0.5 }),
            Rc::new(FlaggedPendingEscalationMappingAdjuster {
                flags: flags.clone(),
                flag: "pending-escalation-mapping-lenient".to_string(),
                percent: 2.2,
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 4036.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_escalation_mapping_engine(&flags::new_registry());
        let item = new_pending_escalation_mapping("code-298", "title-943");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
