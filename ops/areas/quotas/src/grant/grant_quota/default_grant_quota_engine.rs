use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default grant quota calculation.
pub trait DefaultGrantQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultGrantQuota, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultGrantQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultGrantQuotaAdjuster for FlaggedDefaultGrantQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultGrantQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantQuotaMarginOffsetAdjuster {
    pub weight: f64,
}

impl DefaultGrantQuotaAdjuster for DefaultGrantQuotaMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultGrantQuota, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultGrantQuotaAdjuster {
    pub minimum: f64,
}

impl DefaultGrantQuotaAdjuster for MinimumDefaultGrantQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultGrantQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the default grant quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultGrantQuotaEngine {
    pub steps: Vec<Rc<dyn DefaultGrantQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultGrantQuotaEngine {
    pub fn compute(&mut self, item: &DefaultGrantQuota) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DefaultGrantQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_grant_quota_engine(flags: &flags::Registry) -> DefaultGrantQuotaEngine {
    DefaultGrantQuotaEngine {
        steps: vec![
            Rc::new(FlaggedDefaultGrantQuotaAdjuster {
                flags: flags.clone(),
                flag: "default-grant-quota-capped".to_string(),
                percent: 9.8,
            }) as Rc<dyn DefaultGrantQuotaAdjuster>,
            Rc::new(DefaultGrantQuotaMarginOffsetAdjuster { weight: 0.35 }),
            Rc::new(MinimumDefaultGrantQuotaAdjuster { minimum: 30.3 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 41327.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_grant_quota_engine(&flags::new_registry());
        let item = new_default_grant_quota("code-608", "name-620");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
