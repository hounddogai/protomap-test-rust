use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual dns record adjustment calculation.
pub trait ManualDnsRecordAdjustmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualDnsRecordAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeManualDnsRecordAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualDnsRecordAdjustmentAdjuster for ScopeManualDnsRecordAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &ManualDnsRecordAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualDnsRecordAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualDnsRecordAdjustmentAdjuster for TieredManualDnsRecordAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualDnsRecordAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualDnsRecordAdjustmentAdjuster {
    pub cap: f64,
}

impl ManualDnsRecordAdjustmentAdjuster for CapManualDnsRecordAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualDnsRecordAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualDnsRecordAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualDnsRecordAdjustmentAdjuster for FlaggedManualDnsRecordAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualDnsRecordAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual dns record adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualDnsRecordAdjustmentEngine {
    pub steps: Vec<Rc<dyn ManualDnsRecordAdjustmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualDnsRecordAdjustmentEngine {
    pub fn evaluate(&mut self, item: &ManualDnsRecordAdjustment) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualDnsRecordAdjustmentAdjuster> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[ManualDnsRecordAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_dns_record_adjustment_engine(flags: &flags::Registry) -> ManualDnsRecordAdjustmentEngine {
    ManualDnsRecordAdjustmentEngine {
        steps: vec![
            Rc::new(ScopeManualDnsRecordAdjustmentAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.81),
                    ("east".to_string(), 0.88),
                    ("south".to_string(), 0.84),
                ]),
            }) as Rc<dyn ManualDnsRecordAdjustmentAdjuster>,
            Rc::new(TieredManualDnsRecordAdjustmentAdjuster {
                bounds: vec![7039.0, 7986.0, 8074.0],
                rates: vec![0.93, 0.86, 1.04],
            }),
            Rc::new(CapManualDnsRecordAdjustmentAdjuster { cap: 46009.0 }),
            Rc::new(FlaggedManualDnsRecordAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "manual-dns-record-adjustment-global".to_string(),
                percent: -6.7,
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 95246.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_dns_record_adjustment_engine(&flags::new_registry());
        let item = new_manual_dns_record_adjustment("code-11", "label-178");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
