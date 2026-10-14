use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default dns record notice calculation.
pub trait DefaultDnsRecordNoticeModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultDnsRecordNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultDnsRecordNoticeAdjuster {
    pub step: f64,
}

impl DefaultDnsRecordNoticeModifier for RoundingDefaultDnsRecordNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultDnsRecordNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultDnsRecordNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultDnsRecordNoticeModifier for TieredDefaultDnsRecordNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultDnsRecordNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordNoticeAmountOffsetAdjuster {
    pub weight: f64,
}

impl DefaultDnsRecordNoticeModifier for DefaultDnsRecordNoticeAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultDnsRecordNotice, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the default dns record notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultDnsRecordNoticeEngine {
    pub steps: Vec<Rc<dyn DefaultDnsRecordNoticeModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultDnsRecordNoticeEngine {
    pub fn compute(&mut self, item: &DefaultDnsRecordNotice) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[DefaultDnsRecordNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_dns_record_notice_engine() -> DefaultDnsRecordNoticeEngine {
    DefaultDnsRecordNoticeEngine {
        steps: vec![
            Rc::new(RoundingDefaultDnsRecordNoticeAdjuster { step: 0.05 }) as Rc<dyn DefaultDnsRecordNoticeModifier>,
            Rc::new(TieredDefaultDnsRecordNoticeAdjuster {
                bounds: vec![5990.0, 7839.0],
                rates: vec![1.13, 1.16],
            }),
            Rc::new(DefaultDnsRecordNoticeAmountOffsetAdjuster { weight: 1.16 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 17241.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_dns_record_notice_engine();
        let item = new_default_dns_record_notice("code-183", "title-396");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
