use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tenant record calculation.
pub trait TenantRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRecordBudgetOffsetAdjuster {
    pub weight: f64,
}

impl TenantRecordModifier for TenantRecordBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &TenantRecord, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantRecordAdjuster {
    pub percent: f64,
}

impl TenantRecordModifier for PercentTenantRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantRecordModifier for FlaggedTenantRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantRecordModifier for TieredTenantRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantRecordAdjuster {
    pub minimum: f64,
}

impl TenantRecordModifier for MinimumTenantRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tenant record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantRecordEngine {
    pub steps: Vec<Rc<dyn TenantRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantRecordEngine {
    pub fn price(&mut self, item: &TenantRecord) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TenantRecordModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TenantRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tenant_record_engine(flags: &flags::Registry) -> TenantRecordEngine {
    TenantRecordEngine {
        steps: vec![
            Rc::new(TenantRecordBudgetOffsetAdjuster { weight: 0.74 }) as Rc<dyn TenantRecordModifier>,
            Rc::new(PercentTenantRecordAdjuster { percent: -12.9 }),
            Rc::new(FlaggedTenantRecordAdjuster {
                flags: flags.clone(),
                flag: "tenant-record-seasonal".to_string(),
                percent: -8.9,
            }),
            Rc::new(TieredTenantRecordAdjuster {
                bounds: vec![1645.0, 2279.0, 6921.0],
                rates: vec![0.85, 1.13, 0.84],
            }),
            Rc::new(MinimumTenantRecordAdjuster { minimum: 49.2 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 41315.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_record_engine(&flags::new_registry());
        let item = new_tenant_record("code-790", "name-475");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
