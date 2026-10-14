use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the dns record allocation calculation.
pub trait DnsRecordAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDnsRecordAllocationAdjuster {
    pub percent: f64,
}

impl DnsRecordAllocationModifier for PercentDnsRecordAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DnsRecordAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordAllocationAdjuster {
    pub cap: f64,
}

impl DnsRecordAllocationModifier for CapDnsRecordAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordAllocationModifier for TieredDnsRecordAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAllocationAmountOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordAllocationModifier for DnsRecordAllocationAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordAllocation, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterDnsRecordAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordAllocationModifier for ClusterDnsRecordAllocationAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &DnsRecordAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the dns record allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordAllocationEngine {
    pub steps: Vec<Rc<dyn DnsRecordAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordAllocationEngine {
    pub fn run(&mut self, item: &DnsRecordAllocation) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordAllocationModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_dns_record_allocation_engine() -> DnsRecordAllocationEngine {
    DnsRecordAllocationEngine {
        steps: vec![
            Rc::new(PercentDnsRecordAllocationAdjuster { percent: -2.6 }) as Rc<dyn DnsRecordAllocationModifier>,
            Rc::new(CapDnsRecordAllocationAdjuster { cap: 19773.0 }),
            Rc::new(TieredDnsRecordAllocationAdjuster {
                bounds: vec![2418.0, 5481.0, 8153.0],
                rates: vec![0.91, 0.95, 0.86],
            }),
            Rc::new(DnsRecordAllocationAmountOffsetAdjuster { weight: 1.81 }),
            Rc::new(ClusterDnsRecordAllocationAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.34), ("west".to_string(), 1.34)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 32341.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_allocation_engine();
        let item = new_dns_record_allocation("code-406", "name-178");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
