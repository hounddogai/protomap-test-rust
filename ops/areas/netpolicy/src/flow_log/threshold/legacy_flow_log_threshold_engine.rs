use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy flow log threshold calculation.
pub trait LegacyFlowLogThresholdFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyFlowLogThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLegacyFlowLogThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyFlowLogThresholdFactor for LabelLegacyFlowLogThresholdAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LegacyFlowLogThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyFlowLogThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyFlowLogThresholdFactor for FlaggedLegacyFlowLogThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyFlowLogThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyFlowLogThresholdAdjuster {
    pub percent: f64,
}

impl LegacyFlowLogThresholdFactor for PercentLegacyFlowLogThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyFlowLogThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyFlowLogThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyFlowLogThresholdFactor for TieredLegacyFlowLogThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyFlowLogThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyFlowLogThresholdAdjuster {
    pub step: f64,
}

impl LegacyFlowLogThresholdFactor for RoundingLegacyFlowLogThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyFlowLogThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the legacy flow log threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyFlowLogThresholdEngine {
    pub steps: Vec<Rc<dyn LegacyFlowLogThresholdFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyFlowLogThresholdEngine {
    pub fn compute(&mut self, item: &LegacyFlowLogThreshold) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyFlowLogThresholdFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyFlowLogThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_flow_log_threshold_engine(flags: &flags::Registry) -> LegacyFlowLogThresholdEngine {
    LegacyFlowLogThresholdEngine {
        steps: vec![
            Rc::new(LabelLegacyFlowLogThresholdAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.24),
                    ("retail".to_string(), 1.21),
                    ("east".to_string(), 0.96),
                ]),
            }) as Rc<dyn LegacyFlowLogThresholdFactor>,
            Rc::new(FlaggedLegacyFlowLogThresholdAdjuster {
                flags: flags.clone(),
                flag: "legacy-flow-log-threshold-regional".to_string(),
                percent: 3.7,
            }),
            Rc::new(PercentLegacyFlowLogThresholdAdjuster { percent: -14.1 }),
            Rc::new(TieredLegacyFlowLogThresholdAdjuster {
                bounds: vec![5173.0, 9543.0],
                rates: vec![1.06, 0.92],
            }),
            Rc::new(RoundingLegacyFlowLogThresholdAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 86533.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_flow_log_threshold_engine(&flags::new_registry());
        let item = new_legacy_flow_log_threshold("code-179", "label-165");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
