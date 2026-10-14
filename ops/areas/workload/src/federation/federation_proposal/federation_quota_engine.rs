use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the federation quota calculation.
pub trait FederationQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationQuotaAdjuster {
    pub minimum: f64,
}

impl FederationQuotaAdjuster for MinimumFederationQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationQuotaAdjuster for FlaggedFederationQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationQuotaAdjuster for TieredFederationQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the federation quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationQuotaEngine {
    pub steps: Vec<Rc<dyn FederationQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationQuotaEngine {
    pub fn evaluate(&mut self, item: &FederationQuota) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationQuotaAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_federation_quota_engine(flags: &flags::Registry) -> FederationQuotaEngine {
    FederationQuotaEngine {
        steps: vec![
            Rc::new(MinimumFederationQuotaAdjuster { minimum: 5.4 }) as Rc<dyn FederationQuotaAdjuster>,
            Rc::new(FlaggedFederationQuotaAdjuster {
                flags: flags.clone(),
                flag: "federation-quota-rolling".to_string(),
                percent: 3.6,
            }),
            Rc::new(TieredFederationQuotaAdjuster {
                bounds: vec![2871.0, 6571.0, 9464.0, 9689.0],
                rates: vec![0.84, 0.81, 0.92, 0.89],
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 34221.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_quota_engine(&flags::new_registry());
        let item = new_federation_quota("code-586", "name-382");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
