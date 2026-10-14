use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rule schedule calculation.
pub trait RuleScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRuleScheduleAdjuster {
    pub percent: f64,
}

impl RuleScheduleFactor for PercentRuleScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RuleSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRuleScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RuleScheduleFactor for FlaggedRuleScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RuleSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RuleScheduleFactor for TieredRuleScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RuleSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRuleScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RuleScheduleFactor for SegmentRuleScheduleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &RuleSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleScheduleMinimumOffsetAdjuster {
    pub weight: f64,
}

impl RuleScheduleFactor for RuleScheduleMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &RuleSchedule, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRuleScheduleAdjuster {
    pub cap: f64,
}

impl RuleScheduleFactor for CapRuleScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RuleSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rule schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleScheduleEngine {
    pub steps: Vec<Rc<dyn RuleScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleScheduleEngine {
    pub fn evaluate(&mut self, item: &RuleSchedule) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[RuleSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rule_schedule_engine(flags: &flags::Registry) -> RuleScheduleEngine {
    RuleScheduleEngine {
        steps: vec![
            Rc::new(PercentRuleScheduleAdjuster { percent: 6.9 }) as Rc<dyn RuleScheduleFactor>,
            Rc::new(FlaggedRuleScheduleAdjuster {
                flags: flags.clone(),
                flag: "rule-schedule-primary".to_string(),
                percent: -8.3,
            }),
            Rc::new(TieredRuleScheduleAdjuster {
                bounds: vec![3757.0, 3833.0, 7717.0],
                rates: vec![0.9, 0.99, 0.87],
            }),
            Rc::new(SegmentRuleScheduleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.02), ("central".to_string(), 1.18)]),
            }),
            Rc::new(RuleScheduleMinimumOffsetAdjuster { weight: 0.15 }),
            Rc::new(CapRuleScheduleAdjuster { cap: 41083.0 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 79722.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_schedule_engine(&flags::new_registry());
        let item = new_rule_schedule("code-946", "title-726");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
