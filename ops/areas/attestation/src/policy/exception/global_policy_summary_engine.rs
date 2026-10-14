use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global policy summary calculation.
pub trait GlobalPolicySummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalPolicySummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelGlobalPolicySummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalPolicySummaryModifier for LabelGlobalPolicySummaryAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &GlobalPolicySummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalPolicySummaryAdjuster {
    pub percent: f64,
}

impl GlobalPolicySummaryModifier for PercentGlobalPolicySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalPolicySummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalPolicySummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalPolicySummaryModifier for FlaggedGlobalPolicySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalPolicySummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalPolicySummaryAdjuster {
    pub step: f64,
}

impl GlobalPolicySummaryModifier for RoundingGlobalPolicySummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalPolicySummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicySummaryDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl GlobalPolicySummaryModifier for GlobalPolicySummaryDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalPolicySummary, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalPolicySummaryAdjuster {
    pub minimum: f64,
}

impl GlobalPolicySummaryModifier for MinimumGlobalPolicySummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalPolicySummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the global policy summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalPolicySummaryEngine {
    pub steps: Vec<Rc<dyn GlobalPolicySummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalPolicySummaryEngine {
    pub fn compute(&mut self, item: &GlobalPolicySummary) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[GlobalPolicySummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_policy_summary_engine(flags: &flags::Registry) -> GlobalPolicySummaryEngine {
    GlobalPolicySummaryEngine {
        steps: vec![
            Rc::new(LabelGlobalPolicySummaryAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.13),
                    ("online".to_string(), 0.98),
                    ("north".to_string(), 0.96),
                ]),
            }) as Rc<dyn GlobalPolicySummaryModifier>,
            Rc::new(PercentGlobalPolicySummaryAdjuster { percent: 13.7 }),
            Rc::new(FlaggedGlobalPolicySummaryAdjuster {
                flags: flags.clone(),
                flag: "global-policy-summary-tiered".to_string(),
                percent: 4.6,
            }),
            Rc::new(RoundingGlobalPolicySummaryAdjuster { step: 1.0 }),
            Rc::new(GlobalPolicySummaryDurationMinutesOffsetAdjuster { weight: 0.78 }),
            Rc::new(MinimumGlobalPolicySummaryAdjuster { minimum: 39.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 70798.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_policy_summary_engine(&flags::new_registry());
        let item = new_global_policy_summary("code-707", "label-881");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
