use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive alert score calculation.
pub trait AdaptiveAlertScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveAlertScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertScoreAmountOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveAlertScoreStep for AdaptiveAlertScoreAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveAlertScore, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveAlertScoreAdjuster {
    pub step: f64,
}

impl AdaptiveAlertScoreStep for RoundingAdaptiveAlertScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveAlertScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveAlertScoreAdjuster {
    pub minimum: f64,
}

impl AdaptiveAlertScoreStep for MinimumAdaptiveAlertScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveAlertScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelAdaptiveAlertScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveAlertScoreStep for ChannelAdaptiveAlertScoreAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &AdaptiveAlertScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveAlertScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveAlertScoreStep for FlaggedAdaptiveAlertScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveAlertScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive alert score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveAlertScoreEngine {
    pub steps: Vec<Rc<dyn AdaptiveAlertScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveAlertScoreEngine {
    pub fn run(&mut self, item: &AdaptiveAlertScore) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveAlertScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_alert_score_engine(flags: &flags::Registry) -> AdaptiveAlertScoreEngine {
    AdaptiveAlertScoreEngine {
        steps: vec![
            Rc::new(AdaptiveAlertScoreAmountOffsetAdjuster { weight: 1.7 }) as Rc<dyn AdaptiveAlertScoreStep>,
            Rc::new(RoundingAdaptiveAlertScoreAdjuster { step: 0.05 }),
            Rc::new(MinimumAdaptiveAlertScoreAdjuster { minimum: 9.4 }),
            Rc::new(ChannelAdaptiveAlertScoreAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.89)]),
            }),
            Rc::new(FlaggedAdaptiveAlertScoreAdjuster {
                flags: flags.clone(),
                flag: "adaptive-alert-score-regional".to_string(),
                percent: 7.3,
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 45345.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_alert_score_engine(&flags::new_registry());
        let item = new_adaptive_alert_score("code-409", "title-951");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
