use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the escalation notice calculation.
pub trait EscalationNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationNoticeFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl EscalationNoticeStep for EscalationNoticeFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &EscalationNotice, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationNoticeStep for TieredEscalationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEscalationNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EscalationNoticeStep for FlaggedEscalationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EscalationNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEscalationNoticeAdjuster {
    pub minimum: f64,
}

impl EscalationNoticeStep for MinimumEscalationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EscalationNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the escalation notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationNoticeEngine {
    pub steps: Vec<Rc<dyn EscalationNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationNoticeEngine {
    pub fn evaluate(&mut self, item: &EscalationNotice) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[EscalationNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_escalation_notice_engine(flags: &flags::Registry) -> EscalationNoticeEngine {
    EscalationNoticeEngine {
        steps: vec![
            Rc::new(EscalationNoticeFloorValueOffsetAdjuster { weight: 1.77 }) as Rc<dyn EscalationNoticeStep>,
            Rc::new(TieredEscalationNoticeAdjuster {
                bounds: vec![4594.0, 8030.0],
                rates: vec![0.99, 0.92],
            }),
            Rc::new(FlaggedEscalationNoticeAdjuster {
                flags: flags.clone(),
                flag: "escalation-notice-capped".to_string(),
                percent: 1.6,
            }),
            Rc::new(MinimumEscalationNoticeAdjuster { minimum: 32.7 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 40543.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_notice_engine(&flags::new_registry());
        let item = new_escalation_notice("code-418", "label-165");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
