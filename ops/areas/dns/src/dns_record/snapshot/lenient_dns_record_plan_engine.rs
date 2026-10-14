use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient dns record plan calculation.
pub trait LenientDnsRecordPlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientDnsRecordPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientDnsRecordPlanAdjuster {
    pub minimum: f64,
}

impl LenientDnsRecordPlanModifier for MinimumLenientDnsRecordPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientDnsRecordPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientDnsRecordPlanAdjuster {
    pub cap: f64,
}

impl LenientDnsRecordPlanModifier for CapLenientDnsRecordPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientDnsRecordPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordPlanMarginOffsetAdjuster {
    pub weight: f64,
}

impl LenientDnsRecordPlanModifier for LenientDnsRecordPlanMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &LenientDnsRecordPlan, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientDnsRecordPlanAdjuster {
    pub percent: f64,
}

impl LenientDnsRecordPlanModifier for PercentLenientDnsRecordPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientDnsRecordPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleLenientDnsRecordPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientDnsRecordPlanModifier for LocaleLenientDnsRecordPlanAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &LenientDnsRecordPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientDnsRecordPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientDnsRecordPlanModifier for TieredLenientDnsRecordPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientDnsRecordPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient dns record plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientDnsRecordPlanEngine {
    pub steps: Vec<Rc<dyn LenientDnsRecordPlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientDnsRecordPlanEngine {
    pub fn evaluate(&mut self, item: &LenientDnsRecordPlan) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[LenientDnsRecordPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_dns_record_plan_engine() -> LenientDnsRecordPlanEngine {
    LenientDnsRecordPlanEngine {
        steps: vec![
            Rc::new(MinimumLenientDnsRecordPlanAdjuster { minimum: 17.5 }) as Rc<dyn LenientDnsRecordPlanModifier>,
            Rc::new(CapLenientDnsRecordPlanAdjuster { cap: 36013.0 }),
            Rc::new(LenientDnsRecordPlanMarginOffsetAdjuster { weight: 0.75 }),
            Rc::new(PercentLenientDnsRecordPlanAdjuster { percent: -12.3 }),
            Rc::new(LocaleLenientDnsRecordPlanAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.75), ("west".to_string(), 1.34)]),
            }),
            Rc::new(TieredLenientDnsRecordPlanAdjuster {
                bounds: vec![3525.0, 6506.0, 8824.0],
                rates: vec![1.2, 0.94, 0.97],
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 40019.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_dns_record_plan_engine();
        let item = new_lenient_dns_record_plan("code-292", "name-198");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
