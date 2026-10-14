use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the ttl policy record calculation.
pub trait TtlPolicyRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicyRecordAdjuster {
    pub cap: f64,
}

impl TtlPolicyRecordStep for CapTtlPolicyRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicyRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTtlPolicyRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyRecordStep for ChannelTtlPolicyRecordAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &TtlPolicyRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyRecordAdjuster {
    pub percent: f64,
}

impl TtlPolicyRecordStep for PercentTtlPolicyRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyRecordAdjuster {
    pub minimum: f64,
}

impl TtlPolicyRecordStep for MinimumTtlPolicyRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTtlPolicyRecordAdjuster {
    pub step: f64,
}

impl TtlPolicyRecordStep for RoundingTtlPolicyRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TtlPolicyRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyRecordStep for FlaggedTtlPolicyRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the ttl policy record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyRecordEngine {
    pub steps: Vec<Rc<dyn TtlPolicyRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyRecordEngine {
    pub fn run(&mut self, item: &TtlPolicyRecord) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_ttl_policy_record_engine(flags: &flags::Registry) -> TtlPolicyRecordEngine {
    TtlPolicyRecordEngine {
        steps: vec![
            Rc::new(CapTtlPolicyRecordAdjuster { cap: 19544.0 }) as Rc<dyn TtlPolicyRecordStep>,
            Rc::new(ChannelTtlPolicyRecordAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.32)]),
            }),
            Rc::new(PercentTtlPolicyRecordAdjuster { percent: 6.3 }),
            Rc::new(MinimumTtlPolicyRecordAdjuster { minimum: 23.7 }),
            Rc::new(RoundingTtlPolicyRecordAdjuster { step: 5.0 }),
            Rc::new(FlaggedTtlPolicyRecordAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-record-capped".to_string(),
                percent: 9.5,
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 10388.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_record_engine(&flags::new_registry());
        let item = new_ttl_policy_record("code-86", "label-975");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
