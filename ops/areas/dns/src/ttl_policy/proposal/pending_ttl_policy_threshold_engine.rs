use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending ttl policy threshold calculation.
pub trait PendingTtlPolicyThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingTtlPolicyThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingTtlPolicyThresholdAdjuster {
    pub minimum: f64,
}

impl PendingTtlPolicyThresholdStep for MinimumPendingTtlPolicyThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingTtlPolicyThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePendingTtlPolicyThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingTtlPolicyThresholdStep for TitlePendingTtlPolicyThresholdAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PendingTtlPolicyThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingTtlPolicyThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingTtlPolicyThresholdStep for FlaggedPendingTtlPolicyThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingTtlPolicyThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingTtlPolicyThresholdAdjuster {
    pub step: f64,
}

impl PendingTtlPolicyThresholdStep for RoundingPendingTtlPolicyThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingTtlPolicyThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingTtlPolicyThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingTtlPolicyThresholdStep for TieredPendingTtlPolicyThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingTtlPolicyThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingTtlPolicyThresholdAdjuster {
    pub percent: f64,
}

impl PendingTtlPolicyThresholdStep for PercentPendingTtlPolicyThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingTtlPolicyThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending ttl policy threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingTtlPolicyThresholdEngine {
    pub steps: Vec<Rc<dyn PendingTtlPolicyThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingTtlPolicyThresholdEngine {
    pub fn evaluate(&mut self, item: &PendingTtlPolicyThreshold) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[PendingTtlPolicyThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_ttl_policy_threshold_engine(flags: &flags::Registry) -> PendingTtlPolicyThresholdEngine {
    PendingTtlPolicyThresholdEngine {
        steps: vec![
            Rc::new(MinimumPendingTtlPolicyThresholdAdjuster { minimum: 20.6 })
                as Rc<dyn PendingTtlPolicyThresholdStep>,
            Rc::new(TitlePendingTtlPolicyThresholdAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.12),
                    ("retail".to_string(), 1.04),
                    ("west".to_string(), 0.99),
                ]),
            }),
            Rc::new(FlaggedPendingTtlPolicyThresholdAdjuster {
                flags: flags.clone(),
                flag: "pending-ttl-policy-threshold-legacy".to_string(),
                percent: -3.8,
            }),
            Rc::new(RoundingPendingTtlPolicyThresholdAdjuster { step: 0.5 }),
            Rc::new(TieredPendingTtlPolicyThresholdAdjuster {
                bounds: vec![3470.0, 4281.0, 4879.0, 6704.0],
                rates: vec![0.93, 1.05, 0.96, 0.87],
            }),
            Rc::new(PercentPendingTtlPolicyThresholdAdjuster { percent: -16.3 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 74205.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_ttl_policy_threshold_engine(&flags::new_registry());
        let item = new_pending_ttl_policy_threshold("code-51", "title-22");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
