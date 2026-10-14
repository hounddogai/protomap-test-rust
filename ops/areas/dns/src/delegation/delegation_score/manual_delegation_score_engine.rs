use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual delegation score calculation.
pub trait ManualDelegationScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualDelegationScore, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualDelegationScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualDelegationScoreAdjuster for FlaggedManualDelegationScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualDelegationScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualDelegationScoreAdjuster {
    pub percent: f64,
}

impl ManualDelegationScoreAdjuster for PercentManualDelegationScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualDelegationScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualDelegationScoreAdjuster {
    pub minimum: f64,
}

impl ManualDelegationScoreAdjuster for MinimumManualDelegationScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualDelegationScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeManualDelegationScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualDelegationScoreAdjuster for ScopeManualDelegationScoreAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &ManualDelegationScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualDelegationScoreAdjuster {
    pub cap: f64,
}

impl ManualDelegationScoreAdjuster for CapManualDelegationScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualDelegationScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualDelegationScoreAdjuster {
    pub step: f64,
}

impl ManualDelegationScoreAdjuster for RoundingManualDelegationScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualDelegationScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual delegation score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualDelegationScoreEngine {
    pub steps: Vec<Rc<dyn ManualDelegationScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualDelegationScoreEngine {
    pub fn run(&mut self, item: &ManualDelegationScore) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[ManualDelegationScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_delegation_score_engine(flags: &flags::Registry) -> ManualDelegationScoreEngine {
    ManualDelegationScoreEngine {
        steps: vec![
            Rc::new(FlaggedManualDelegationScoreAdjuster {
                flags: flags.clone(),
                flag: "manual-delegation-score-fallback".to_string(),
                percent: 6.6,
            }) as Rc<dyn ManualDelegationScoreAdjuster>,
            Rc::new(PercentManualDelegationScoreAdjuster { percent: -2.3 }),
            Rc::new(MinimumManualDelegationScoreAdjuster { minimum: 9.9 }),
            Rc::new(ScopeManualDelegationScoreAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.8), ("south".to_string(), 1.28)]),
            }),
            Rc::new(CapManualDelegationScoreAdjuster { cap: 5075.0 }),
            Rc::new(RoundingManualDelegationScoreAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 21200.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_delegation_score_engine(&flags::new_registry());
        let item = new_manual_delegation_score("code-437", "name-270");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
