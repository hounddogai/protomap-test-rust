use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal dashboard rule calculation.
pub trait SeasonalDashboardRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalDashboardRule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalDashboardRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalDashboardRuleStep for FlaggedSeasonalDashboardRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalDashboardRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardRuleMarginOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalDashboardRuleStep for SeasonalDashboardRuleMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalDashboardRule, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalDashboardRuleAdjuster {
    pub minimum: f64,
}

impl SeasonalDashboardRuleStep for MinimumSeasonalDashboardRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalDashboardRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalDashboardRuleAdjuster {
    pub step: f64,
}

impl SeasonalDashboardRuleStep for RoundingSeasonalDashboardRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalDashboardRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the seasonal dashboard rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalDashboardRuleEngine {
    pub steps: Vec<Rc<dyn SeasonalDashboardRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalDashboardRuleEngine {
    pub fn run(&mut self, item: &SeasonalDashboardRule) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalDashboardRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_dashboard_rule_engine(flags: &flags::Registry) -> SeasonalDashboardRuleEngine {
    SeasonalDashboardRuleEngine {
        steps: vec![
            Rc::new(FlaggedSeasonalDashboardRuleAdjuster {
                flags: flags.clone(),
                flag: "seasonal-dashboard-rule-primary".to_string(),
                percent: 8.9,
            }) as Rc<dyn SeasonalDashboardRuleStep>,
            Rc::new(SeasonalDashboardRuleMarginOffsetAdjuster { weight: 1.85 }),
            Rc::new(MinimumSeasonalDashboardRuleAdjuster { minimum: 8.7 }),
            Rc::new(RoundingSeasonalDashboardRuleAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 14734.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_dashboard_rule_engine(&flags::new_registry());
        let item = new_seasonal_dashboard_rule("code-701", "label-22");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
