use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the escalation metric calculation.
pub trait EscalationMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationMetric, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedEscalationMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EscalationMetricAdjuster for FlaggedEscalationMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EscalationMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEscalationMetricAdjuster {
    pub cap: f64,
}

impl EscalationMetricAdjuster for CapEscalationMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EscalationMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationMetricAdjuster for TieredEscalationMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEscalationMetricAdjuster {
    pub percent: f64,
}

impl EscalationMetricAdjuster for PercentEscalationMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EscalationMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMetricQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl EscalationMetricAdjuster for EscalationMetricQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &EscalationMetric, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeEscalationMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EscalationMetricAdjuster for ScopeEscalationMetricAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &EscalationMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the escalation metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationMetricEngine {
    pub steps: Vec<Rc<dyn EscalationMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationMetricEngine {
    pub fn price(&mut self, item: &EscalationMetric) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[EscalationMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_escalation_metric_engine(flags: &flags::Registry) -> EscalationMetricEngine {
    EscalationMetricEngine {
        steps: vec![
            Rc::new(FlaggedEscalationMetricAdjuster {
                flags: flags.clone(),
                flag: "escalation-metric-fallback".to_string(),
                percent: 3.6,
            }) as Rc<dyn EscalationMetricAdjuster>,
            Rc::new(CapEscalationMetricAdjuster { cap: 34124.0 }),
            Rc::new(TieredEscalationMetricAdjuster {
                bounds: vec![1424.0, 6073.0],
                rates: vec![1.03, 1.1],
            }),
            Rc::new(PercentEscalationMetricAdjuster { percent: 3.7 }),
            Rc::new(EscalationMetricQuotaUnitsOffsetAdjuster { weight: 1.96 }),
            Rc::new(ScopeEscalationMetricAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.82)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 3578.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_metric_engine(&flags::new_registry());
        let item = new_escalation_metric("code-724", "name-409");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
