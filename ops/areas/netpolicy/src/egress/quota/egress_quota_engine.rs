use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the egress quota calculation.
pub trait EgressQuotaStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressQuotaAdjuster {
    pub percent: f64,
}

impl EgressQuotaStep for PercentEgressQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EgressQuotaStep for TieredEgressQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EgressQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressQuotaEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl EgressQuotaStep for EgressQuotaEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &EgressQuota, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressQuotaAdjuster {
    pub step: f64,
}

impl EgressQuotaStep for RoundingEgressQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEgressQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EgressQuotaStep for FlaggedEgressQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EgressQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the egress quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressQuotaEngine {
    pub steps: Vec<Rc<dyn EgressQuotaStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressQuotaEngine {
    pub fn evaluate(&mut self, item: &EgressQuota) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[EgressQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_egress_quota_engine(flags: &flags::Registry) -> EgressQuotaEngine {
    EgressQuotaEngine {
        steps: vec![
            Rc::new(PercentEgressQuotaAdjuster { percent: 11.2 }) as Rc<dyn EgressQuotaStep>,
            Rc::new(TieredEgressQuotaAdjuster {
                bounds: vec![3874.0, 7531.0],
                rates: vec![1.01, 0.86],
            }),
            Rc::new(EgressQuotaEndsAtOffsetAdjuster { weight: 1.79 }),
            Rc::new(RoundingEgressQuotaAdjuster { step: 0.5 }),
            Rc::new(FlaggedEgressQuotaAdjuster {
                flags: flags.clone(),
                flag: "egress-quota-strict".to_string(),
                percent: 4.7,
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 54030.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_quota_engine(&flags::new_registry());
        let item = new_egress_quota("code-455", "label-959");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
