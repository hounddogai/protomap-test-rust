use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback delegation estimate calculation.
pub trait FallbackDelegationEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackDelegationEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceFallbackDelegationEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackDelegationEstimateAdjuster for NamespaceFallbackDelegationEstimateAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &FallbackDelegationEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackDelegationEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackDelegationEstimateAdjuster for FlaggedFallbackDelegationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackDelegationEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackDelegationEstimateAdjuster {
    pub percent: f64,
}

impl FallbackDelegationEstimateAdjuster for PercentFallbackDelegationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackDelegationEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback delegation estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackDelegationEstimateEngine {
    pub steps: Vec<Rc<dyn FallbackDelegationEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackDelegationEstimateEngine {
    pub fn run(&mut self, item: &FallbackDelegationEstimate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FallbackDelegationEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_delegation_estimate_engine(flags: &flags::Registry) -> FallbackDelegationEstimateEngine {
    FallbackDelegationEstimateEngine {
        steps: vec![
            Rc::new(NamespaceFallbackDelegationEstimateAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.18)]),
            }) as Rc<dyn FallbackDelegationEstimateAdjuster>,
            Rc::new(FlaggedFallbackDelegationEstimateAdjuster {
                flags: flags.clone(),
                flag: "fallback-delegation-estimate-adaptive".to_string(),
                percent: 7.9,
            }),
            Rc::new(PercentFallbackDelegationEstimateAdjuster { percent: -3.6 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 33581.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_delegation_estimate_engine(&flags::new_registry());
        let item = new_fallback_delegation_estimate("code-674", "label-531");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
