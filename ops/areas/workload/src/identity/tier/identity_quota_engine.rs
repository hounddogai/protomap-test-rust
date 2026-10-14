use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the identity quota calculation.
pub trait IdentityQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityQuotaAdjuster {
    pub cap: f64,
}

impl IdentityQuotaAdjuster for CapIdentityQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceIdentityQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityQuotaAdjuster for SourceIdentityQuotaAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &IdentityQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityQuotaAdjuster for TieredIdentityQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the identity quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityQuotaEngine {
    pub steps: Vec<Rc<dyn IdentityQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityQuotaEngine {
    pub fn evaluate(&mut self, item: &IdentityQuota) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[IdentityQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_identity_quota_engine() -> IdentityQuotaEngine {
    IdentityQuotaEngine {
        steps: vec![
            Rc::new(CapIdentityQuotaAdjuster { cap: 46757.0 }) as Rc<dyn IdentityQuotaAdjuster>,
            Rc::new(SourceIdentityQuotaAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.89)]),
            }),
            Rc::new(TieredIdentityQuotaAdjuster {
                bounds: vec![1370.0, 1751.0, 3634.0, 7814.0],
                rates: vec![1.03, 1.01, 0.85, 0.95],
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 66669.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_quota_engine();
        let item = new_identity_quota("code-79", "name-734");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
