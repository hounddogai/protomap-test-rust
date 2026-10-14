use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled delegation score calculation.
pub trait ScheduledDelegationScoreFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledDelegationScore, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledDelegationScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledDelegationScoreFactor for FlaggedScheduledDelegationScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledDelegationScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledDelegationScoreAdjuster {
    pub percent: f64,
}

impl ScheduledDelegationScoreFactor for PercentScheduledDelegationScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledDelegationScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDelegationScoreRatioOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledDelegationScoreFactor for ScheduledDelegationScoreRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledDelegationScore, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledDelegationScoreAdjuster {
    pub cap: f64,
}

impl ScheduledDelegationScoreFactor for CapScheduledDelegationScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledDelegationScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelScheduledDelegationScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledDelegationScoreFactor for ChannelScheduledDelegationScoreAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &ScheduledDelegationScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled delegation score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledDelegationScoreEngine {
    pub steps: Vec<Rc<dyn ScheduledDelegationScoreFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledDelegationScoreEngine {
    pub fn run(&mut self, item: &ScheduledDelegationScore) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ScheduledDelegationScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_delegation_score_engine(flags: &flags::Registry) -> ScheduledDelegationScoreEngine {
    ScheduledDelegationScoreEngine {
        steps: vec![
            Rc::new(FlaggedScheduledDelegationScoreAdjuster {
                flags: flags.clone(),
                flag: "scheduled-delegation-score-tiered".to_string(),
                percent: -8.3,
            }) as Rc<dyn ScheduledDelegationScoreFactor>,
            Rc::new(PercentScheduledDelegationScoreAdjuster { percent: 20.9 }),
            Rc::new(ScheduledDelegationScoreRatioOffsetAdjuster { weight: 1.33 }),
            Rc::new(CapScheduledDelegationScoreAdjuster { cap: 47650.0 }),
            Rc::new(ChannelScheduledDelegationScoreAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.09),
                    ("online".to_string(), 0.83),
                    ("retail".to_string(), 1.02),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 22525.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_delegation_score_engine(&flags::new_registry());
        let item = new_scheduled_delegation_score("code-928", "label-443");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
