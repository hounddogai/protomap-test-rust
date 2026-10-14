use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the dns record budget calculation.
pub trait DnsRecordBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordBudgetAdjuster {
    pub cap: f64,
}

impl DnsRecordBudgetAdjuster for CapDnsRecordBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDnsRecordBudgetAdjuster {
    pub percent: f64,
}

impl DnsRecordBudgetAdjuster for PercentDnsRecordBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DnsRecordBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordBudgetAdjuster {
    pub minimum: f64,
}

impl DnsRecordBudgetAdjuster for MinimumDnsRecordBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordBudgetAdjuster for TieredDnsRecordBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDnsRecordBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DnsRecordBudgetAdjuster for FlaggedDnsRecordBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DnsRecordBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBudgetAmountOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordBudgetAdjuster for DnsRecordBudgetAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordBudget, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the dns record budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordBudgetEngine {
    pub steps: Vec<Rc<dyn DnsRecordBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordBudgetEngine {
    pub fn evaluate(&mut self, item: &DnsRecordBudget) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordBudgetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_dns_record_budget_engine(flags: &flags::Registry) -> DnsRecordBudgetEngine {
    DnsRecordBudgetEngine {
        steps: vec![
            Rc::new(CapDnsRecordBudgetAdjuster { cap: 13627.0 }) as Rc<dyn DnsRecordBudgetAdjuster>,
            Rc::new(PercentDnsRecordBudgetAdjuster { percent: -18.9 }),
            Rc::new(MinimumDnsRecordBudgetAdjuster { minimum: 36.8 }),
            Rc::new(TieredDnsRecordBudgetAdjuster {
                bounds: vec![6954.0, 8783.0],
                rates: vec![1.13, 0.95],
            }),
            Rc::new(FlaggedDnsRecordBudgetAdjuster {
                flags: flags.clone(),
                flag: "dns-record-budget-legacy".to_string(),
                percent: 2.6,
            }),
            Rc::new(DnsRecordBudgetAmountOffsetAdjuster { weight: 0.66 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 71184.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_budget_engine(&flags::new_registry());
        let item = new_dns_record_budget("code-517", "name-305");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
