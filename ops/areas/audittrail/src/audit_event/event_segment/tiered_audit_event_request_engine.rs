use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered audit event request calculation.
pub trait TieredAuditEventRequestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredAuditEventRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventRequestBudgetOffsetAdjuster {
    pub weight: f64,
}

impl TieredAuditEventRequestAdjuster for TieredAuditEventRequestBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &TieredAuditEventRequest, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredAuditEventRequestAdjuster {
    pub percent: f64,
}

impl TieredAuditEventRequestAdjuster for PercentTieredAuditEventRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredAuditEventRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredAuditEventRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredAuditEventRequestAdjuster for TieredTieredAuditEventRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredAuditEventRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredAuditEventRequestAdjuster {
    pub step: f64,
}

impl TieredAuditEventRequestAdjuster for RoundingTieredAuditEventRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredAuditEventRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredAuditEventRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredAuditEventRequestAdjuster for FlaggedTieredAuditEventRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredAuditEventRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered audit event request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredAuditEventRequestEngine {
    pub steps: Vec<Rc<dyn TieredAuditEventRequestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredAuditEventRequestEngine {
    pub fn compute(&mut self, item: &TieredAuditEventRequest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TieredAuditEventRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_audit_event_request_engine(flags: &flags::Registry) -> TieredAuditEventRequestEngine {
    TieredAuditEventRequestEngine {
        steps: vec![
            Rc::new(TieredAuditEventRequestBudgetOffsetAdjuster { weight: 0.63 })
                as Rc<dyn TieredAuditEventRequestAdjuster>,
            Rc::new(PercentTieredAuditEventRequestAdjuster { percent: -4.0 }),
            Rc::new(TieredTieredAuditEventRequestAdjuster {
                bounds: vec![2320.0, 4587.0, 6664.0, 7802.0],
                rates: vec![1.19, 1.05, 1.15, 1.04],
            }),
            Rc::new(RoundingTieredAuditEventRequestAdjuster { step: 0.5 }),
            Rc::new(FlaggedTieredAuditEventRequestAdjuster {
                flags: flags.clone(),
                flag: "tiered-audit-event-request-adaptive".to_string(),
                percent: 5.0,
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 64548.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_audit_event_request_engine(&flags::new_registry());
        let item = new_tiered_audit_event_request("code-898", "label-113");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
