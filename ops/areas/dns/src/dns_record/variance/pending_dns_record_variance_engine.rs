use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending dns record variance calculation.
pub trait PendingDnsRecordVarianceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingDnsRecordVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingDnsRecordVarianceAdjuster {
    pub step: f64,
}

impl PendingDnsRecordVarianceFactor for RoundingPendingDnsRecordVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingDnsRecordVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePendingDnsRecordVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingDnsRecordVarianceFactor for SourcePendingDnsRecordVarianceAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PendingDnsRecordVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingDnsRecordVarianceAdjuster {
    pub cap: f64,
}

impl PendingDnsRecordVarianceFactor for CapPendingDnsRecordVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingDnsRecordVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingDnsRecordVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingDnsRecordVarianceFactor for TieredPendingDnsRecordVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingDnsRecordVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingDnsRecordVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingDnsRecordVarianceFactor for FlaggedPendingDnsRecordVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingDnsRecordVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDnsRecordVarianceDiscountOffsetAdjuster {
    pub weight: f64,
}

impl PendingDnsRecordVarianceFactor for PendingDnsRecordVarianceDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &PendingDnsRecordVariance, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

/// Runs the pending dns record variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingDnsRecordVarianceEngine {
    pub steps: Vec<Rc<dyn PendingDnsRecordVarianceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingDnsRecordVarianceEngine {
    pub fn price(&mut self, item: &PendingDnsRecordVariance) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[PendingDnsRecordVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_dns_record_variance_engine(flags: &flags::Registry) -> PendingDnsRecordVarianceEngine {
    PendingDnsRecordVarianceEngine {
        steps: vec![
            Rc::new(RoundingPendingDnsRecordVarianceAdjuster { step: 5.0 }) as Rc<dyn PendingDnsRecordVarianceFactor>,
            Rc::new(SourcePendingDnsRecordVarianceAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.82),
                    ("east".to_string(), 1.3),
                    ("west".to_string(), 0.87),
                ]),
            }),
            Rc::new(CapPendingDnsRecordVarianceAdjuster { cap: 6540.0 }),
            Rc::new(TieredPendingDnsRecordVarianceAdjuster {
                bounds: vec![3053.0, 8836.0],
                rates: vec![1.06, 0.87],
            }),
            Rc::new(FlaggedPendingDnsRecordVarianceAdjuster {
                flags: flags.clone(),
                flag: "pending-dns-record-variance-adaptive".to_string(),
                percent: -0.2,
            }),
            Rc::new(PendingDnsRecordVarianceDiscountOffsetAdjuster { weight: 1.3 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 55739.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_dns_record_variance_engine(&flags::new_registry());
        let item = new_pending_dns_record_variance("code-907", "title-730");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
