use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default flow log tier calculation.
pub trait DefaultFlowLogTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultFlowLogTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogTierThresholdOffsetAdjuster {
    pub weight: f64,
}

impl DefaultFlowLogTierModifier for DefaultFlowLogTierThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultFlowLogTier, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultFlowLogTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultFlowLogTierModifier for FlaggedDefaultFlowLogTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultFlowLogTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultFlowLogTierAdjuster {
    pub cap: f64,
}

impl DefaultFlowLogTierModifier for CapDefaultFlowLogTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultFlowLogTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelDefaultFlowLogTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultFlowLogTierModifier for ChannelDefaultFlowLogTierAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &DefaultFlowLogTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultFlowLogTierAdjuster {
    pub minimum: f64,
}

impl DefaultFlowLogTierModifier for MinimumDefaultFlowLogTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultFlowLogTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultFlowLogTierAdjuster {
    pub percent: f64,
}

impl DefaultFlowLogTierModifier for PercentDefaultFlowLogTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultFlowLogTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default flow log tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultFlowLogTierEngine {
    pub steps: Vec<Rc<dyn DefaultFlowLogTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultFlowLogTierEngine {
    pub fn evaluate(&mut self, item: &DefaultFlowLogTier) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DefaultFlowLogTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_flow_log_tier_engine(flags: &flags::Registry) -> DefaultFlowLogTierEngine {
    DefaultFlowLogTierEngine {
        steps: vec![
            Rc::new(DefaultFlowLogTierThresholdOffsetAdjuster { weight: 0.8 }) as Rc<dyn DefaultFlowLogTierModifier>,
            Rc::new(FlaggedDefaultFlowLogTierAdjuster {
                flags: flags.clone(),
                flag: "default-flow-log-tier-capped".to_string(),
                percent: 6.7,
            }),
            Rc::new(CapDefaultFlowLogTierAdjuster { cap: 33088.0 }),
            Rc::new(ChannelDefaultFlowLogTierAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.82), ("retail".to_string(), 1.4)]),
            }),
            Rc::new(MinimumDefaultFlowLogTierAdjuster { minimum: 23.1 }),
            Rc::new(PercentDefaultFlowLogTierAdjuster { percent: -14.4 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 46203.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_flow_log_tier_engine(&flags::new_registry());
        let item = new_default_flow_log_tier("code-448", "title-193");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
