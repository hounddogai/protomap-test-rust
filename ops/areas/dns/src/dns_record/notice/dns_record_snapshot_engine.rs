use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the dns record snapshot calculation.
pub trait DnsRecordSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDnsRecordSnapshotAdjuster {
    pub percent: f64,
}

impl DnsRecordSnapshotFactor for PercentDnsRecordSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DnsRecordSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordSnapshotAdjuster {
    pub cap: f64,
}

impl DnsRecordSnapshotFactor for CapDnsRecordSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordSnapshotFactor for TieredDnsRecordSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the dns record snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordSnapshotEngine {
    pub steps: Vec<Rc<dyn DnsRecordSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordSnapshotEngine {
    pub fn compute(&mut self, item: &DnsRecordSnapshot) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordSnapshotFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_dns_record_snapshot_engine() -> DnsRecordSnapshotEngine {
    DnsRecordSnapshotEngine {
        steps: vec![
            Rc::new(PercentDnsRecordSnapshotAdjuster { percent: 7.7 }) as Rc<dyn DnsRecordSnapshotFactor>,
            Rc::new(CapDnsRecordSnapshotAdjuster { cap: 32076.0 }),
            Rc::new(TieredDnsRecordSnapshotAdjuster {
                bounds: vec![693.0, 849.0, 1859.0, 2244.0],
                rates: vec![1.17, 0.99, 1.1, 1.19],
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 79349.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_snapshot_engine();
        let item = new_dns_record_snapshot("code-838", "label-338");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
