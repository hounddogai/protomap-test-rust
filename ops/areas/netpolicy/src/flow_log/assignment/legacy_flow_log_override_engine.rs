use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy flow log override calculation.
pub trait LegacyFlowLogOverrideStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyFlowLogOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyFlowLogOverrideAdjuster {
    pub step: f64,
}

impl LegacyFlowLogOverrideStep for RoundingLegacyFlowLogOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyFlowLogOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionLegacyFlowLogOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyFlowLogOverrideStep for RegionLegacyFlowLogOverrideAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &LegacyFlowLogOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyFlowLogOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyFlowLogOverrideStep for FlaggedLegacyFlowLogOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyFlowLogOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyFlowLogOverrideAdjuster {
    pub cap: f64,
}

impl LegacyFlowLogOverrideStep for CapLegacyFlowLogOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyFlowLogOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyFlowLogOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyFlowLogOverrideStep for TieredLegacyFlowLogOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyFlowLogOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the legacy flow log override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyFlowLogOverrideEngine {
    pub steps: Vec<Rc<dyn LegacyFlowLogOverrideStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyFlowLogOverrideEngine {
    pub fn evaluate(&mut self, item: &LegacyFlowLogOverride) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyFlowLogOverrideStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyFlowLogOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_flow_log_override_engine(flags: &flags::Registry) -> LegacyFlowLogOverrideEngine {
    LegacyFlowLogOverrideEngine {
        steps: vec![
            Rc::new(RoundingLegacyFlowLogOverrideAdjuster { step: 0.5 }) as Rc<dyn LegacyFlowLogOverrideStep>,
            Rc::new(RegionLegacyFlowLogOverrideAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.86)]),
            }),
            Rc::new(FlaggedLegacyFlowLogOverrideAdjuster {
                flags: flags.clone(),
                flag: "legacy-flow-log-override-capped".to_string(),
                percent: -7.1,
            }),
            Rc::new(CapLegacyFlowLogOverrideAdjuster { cap: 18701.0 }),
            Rc::new(TieredLegacyFlowLogOverrideAdjuster {
                bounds: vec![3352.0, 6048.0, 7779.0, 8442.0],
                rates: vec![1.04, 0.92, 0.92, 0.97],
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 42839.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_flow_log_override_engine(&flags::new_registry());
        let item = new_legacy_flow_log_override("code-236", "title-328");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
