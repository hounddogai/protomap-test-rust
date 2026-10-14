use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy federation adjustment calculation.
pub trait LegacyFederationAdjustmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyFederationAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyFederationAdjustmentAdjuster {
    pub percent: f64,
}

impl LegacyFederationAdjustmentStep for PercentLegacyFederationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyFederationAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAdjustmentRetriesOffsetAdjuster {
    pub weight: f64,
}

impl LegacyFederationAdjustmentStep for LegacyFederationAdjustmentRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyFederationAdjustment, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyFederationAdjustmentAdjuster {
    pub cap: f64,
}

impl LegacyFederationAdjustmentStep for CapLegacyFederationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyFederationAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyFederationAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyFederationAdjustmentStep for FlaggedLegacyFederationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyFederationAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyFederationAdjustmentAdjuster {
    pub step: f64,
}

impl LegacyFederationAdjustmentStep for RoundingLegacyFederationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyFederationAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentLegacyFederationAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyFederationAdjustmentStep for SegmentLegacyFederationAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &LegacyFederationAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy federation adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyFederationAdjustmentEngine {
    pub steps: Vec<Rc<dyn LegacyFederationAdjustmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyFederationAdjustmentEngine {
    pub fn run(&mut self, item: &LegacyFederationAdjustment) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyFederationAdjustmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyFederationAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_federation_adjustment_engine(flags: &flags::Registry) -> LegacyFederationAdjustmentEngine {
    LegacyFederationAdjustmentEngine {
        steps: vec![
            Rc::new(PercentLegacyFederationAdjustmentAdjuster { percent: 10.3 })
                as Rc<dyn LegacyFederationAdjustmentStep>,
            Rc::new(LegacyFederationAdjustmentRetriesOffsetAdjuster { weight: 1.88 }),
            Rc::new(CapLegacyFederationAdjustmentAdjuster { cap: 45308.0 }),
            Rc::new(FlaggedLegacyFederationAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "legacy-federation-adjustment-primary".to_string(),
                percent: -3.8,
            }),
            Rc::new(RoundingLegacyFederationAdjustmentAdjuster { step: 0.1 }),
            Rc::new(SegmentLegacyFederationAdjustmentAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.83), ("retail".to_string(), 1.32)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 95605.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_federation_adjustment_engine(&flags::new_registry());
        let item = new_legacy_federation_adjustment("code-138", "name-330");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
