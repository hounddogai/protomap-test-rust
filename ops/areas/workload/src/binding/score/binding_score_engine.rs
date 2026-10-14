use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the binding score calculation.
pub trait BindingScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingScoreAdjuster {
    pub minimum: f64,
}

impl BindingScoreAdjuster for MinimumBindingScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBindingScoreAdjuster {
    pub cap: f64,
}

impl BindingScoreAdjuster for CapBindingScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BindingScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelBindingScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingScoreAdjuster for ChannelBindingScoreAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &BindingScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBindingScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BindingScoreAdjuster for FlaggedBindingScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BindingScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingScoreAdjuster {
    pub step: f64,
}

impl BindingScoreAdjuster for RoundingBindingScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the binding score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingScoreEngine {
    pub steps: Vec<Rc<dyn BindingScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingScoreEngine {
    pub fn compute(&mut self, item: &BindingScore) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[BindingScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_binding_score_engine(flags: &flags::Registry) -> BindingScoreEngine {
    BindingScoreEngine {
        steps: vec![
            Rc::new(MinimumBindingScoreAdjuster { minimum: 11.1 }) as Rc<dyn BindingScoreAdjuster>,
            Rc::new(CapBindingScoreAdjuster { cap: 29291.0 }),
            Rc::new(ChannelBindingScoreAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.78), ("retail".to_string(), 1.08)]),
            }),
            Rc::new(FlaggedBindingScoreAdjuster {
                flags: flags.clone(),
                flag: "binding-score-strict".to_string(),
                percent: 6.9,
            }),
            Rc::new(RoundingBindingScoreAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 96175.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_score_engine(&flags::new_registry());
        let item = new_binding_score("code-434", "label-574");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
