use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual redaction target calculation.
pub trait ManualRedactionTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualRedactionTarget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualRedactionTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualRedactionTargetStep for FlaggedManualRedactionTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualRedactionTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualRedactionTargetAdjuster {
    pub percent: f64,
}

impl ManualRedactionTargetStep for PercentManualRedactionTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualRedactionTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualRedactionTargetAdjuster {
    pub step: f64,
}

impl ManualRedactionTargetStep for RoundingManualRedactionTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualRedactionTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionTargetLimitOffsetAdjuster {
    pub weight: f64,
}

impl ManualRedactionTargetStep for ManualRedactionTargetLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &ManualRedactionTarget, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualRedactionTargetAdjuster {
    pub minimum: f64,
}

impl ManualRedactionTargetStep for MinimumManualRedactionTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualRedactionTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual redaction target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualRedactionTargetEngine {
    pub steps: Vec<Rc<dyn ManualRedactionTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualRedactionTargetEngine {
    pub fn price(&mut self, item: &ManualRedactionTarget) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualRedactionTargetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualRedactionTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_redaction_target_engine(flags: &flags::Registry) -> ManualRedactionTargetEngine {
    ManualRedactionTargetEngine {
        steps: vec![
            Rc::new(FlaggedManualRedactionTargetAdjuster {
                flags: flags.clone(),
                flag: "manual-redaction-target-legacy".to_string(),
                percent: -8.8,
            }) as Rc<dyn ManualRedactionTargetStep>,
            Rc::new(PercentManualRedactionTargetAdjuster { percent: 21.8 }),
            Rc::new(RoundingManualRedactionTargetAdjuster { step: 5.0 }),
            Rc::new(ManualRedactionTargetLimitOffsetAdjuster { weight: 0.09 }),
            Rc::new(MinimumManualRedactionTargetAdjuster { minimum: 1.1 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 23537.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_redaction_target_engine(&flags::new_registry());
        let item = new_manual_redaction_target("code-393", "label-102");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
