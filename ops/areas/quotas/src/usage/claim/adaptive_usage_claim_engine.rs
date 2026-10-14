use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive usage claim calculation.
pub trait AdaptiveUsageClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveUsageClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageClaimRevisionOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveUsageClaimModifier for AdaptiveUsageClaimRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveUsageClaim, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveUsageClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveUsageClaimModifier for FlaggedAdaptiveUsageClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveUsageClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveUsageClaimAdjuster {
    pub step: f64,
}

impl AdaptiveUsageClaimModifier for RoundingAdaptiveUsageClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveUsageClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the adaptive usage claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveUsageClaimEngine {
    pub steps: Vec<Rc<dyn AdaptiveUsageClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveUsageClaimEngine {
    pub fn evaluate(&mut self, item: &AdaptiveUsageClaim) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveUsageClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_usage_claim_engine(flags: &flags::Registry) -> AdaptiveUsageClaimEngine {
    AdaptiveUsageClaimEngine {
        steps: vec![
            Rc::new(AdaptiveUsageClaimRevisionOffsetAdjuster { weight: 0.07 }) as Rc<dyn AdaptiveUsageClaimModifier>,
            Rc::new(FlaggedAdaptiveUsageClaimAdjuster {
                flags: flags.clone(),
                flag: "adaptive-usage-claim-pending".to_string(),
                percent: 5.5,
            }),
            Rc::new(RoundingAdaptiveUsageClaimAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 94429.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_usage_claim_engine(&flags::new_registry());
        let item = new_adaptive_usage_claim("code-378", "name-953");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
