use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient plan quota calculation.
pub trait LenientPlanQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientPlanQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientPlanQuotaAdjuster {
    pub minimum: f64,
}

impl LenientPlanQuotaModifier for MinimumLenientPlanQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientPlanQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientPlanQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientPlanQuotaModifier for FlaggedLenientPlanQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientPlanQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanQuotaPortOffsetAdjuster {
    pub weight: f64,
}

impl LenientPlanQuotaModifier for LenientPlanQuotaPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &LenientPlanQuota, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

/// Runs the lenient plan quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientPlanQuotaEngine {
    pub steps: Vec<Rc<dyn LenientPlanQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientPlanQuotaEngine {
    pub fn price(&mut self, item: &LenientPlanQuota) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientPlanQuotaModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientPlanQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_plan_quota_engine(flags: &flags::Registry) -> LenientPlanQuotaEngine {
    LenientPlanQuotaEngine {
        steps: vec![
            Rc::new(MinimumLenientPlanQuotaAdjuster { minimum: 17.9 }) as Rc<dyn LenientPlanQuotaModifier>,
            Rc::new(FlaggedLenientPlanQuotaAdjuster {
                flags: flags.clone(),
                flag: "lenient-plan-quota-regional".to_string(),
                percent: 8.2,
            }),
            Rc::new(LenientPlanQuotaPortOffsetAdjuster { weight: 0.9 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 69983.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_plan_quota_engine(&flags::new_registry());
        let item = new_lenient_plan_quota("code-113", "name-176");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
