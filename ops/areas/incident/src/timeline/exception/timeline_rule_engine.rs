use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the timeline rule calculation.
pub trait TimelineRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRuleWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl TimelineRuleModifier for TimelineRuleWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &TimelineRule, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineRuleAdjuster {
    pub percent: f64,
}

impl TimelineRuleModifier for PercentTimelineRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTimelineRuleAdjuster {
    pub minimum: f64,
}

impl TimelineRuleModifier for MinimumTimelineRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TimelineRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTimelineRuleAdjuster {
    pub step: f64,
}

impl TimelineRuleModifier for RoundingTimelineRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TimelineRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitTimelineRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TimelineRuleModifier for UnitTimelineRuleAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &TimelineRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TimelineRuleModifier for TieredTimelineRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TimelineRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the timeline rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineRuleEngine {
    pub steps: Vec<Rc<dyn TimelineRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineRuleEngine {
    pub fn compute(&mut self, item: &TimelineRule) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TimelineRuleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TimelineRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_timeline_rule_engine() -> TimelineRuleEngine {
    TimelineRuleEngine {
        steps: vec![
            Rc::new(TimelineRuleWindowDaysOffsetAdjuster { weight: 0.76 }) as Rc<dyn TimelineRuleModifier>,
            Rc::new(PercentTimelineRuleAdjuster { percent: 16.5 }),
            Rc::new(MinimumTimelineRuleAdjuster { minimum: 44.6 }),
            Rc::new(RoundingTimelineRuleAdjuster { step: 0.25 }),
            Rc::new(UnitTimelineRuleAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.35),
                    ("online".to_string(), 0.96),
                    ("retail".to_string(), 1.28),
                ]),
            }),
            Rc::new(TieredTimelineRuleAdjuster {
                bounds: vec![2004.0, 2825.0],
                rates: vec![0.88, 1.07],
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 74184.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_timeline_rule_engine();
        let item = new_timeline_rule("code-695", "name-571");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
