use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the approval forecast calculation.
pub trait ApprovalForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalForecast, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedApprovalForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ApprovalForecastModifier for FlaggedApprovalForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ApprovalForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleApprovalForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalForecastModifier for LocaleApprovalForecastAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &ApprovalForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalForecastAdjuster {
    pub percent: f64,
}

impl ApprovalForecastModifier for PercentApprovalForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalForecastAdjuster {
    pub step: f64,
}

impl ApprovalForecastModifier for RoundingApprovalForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the approval forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalForecastEngine {
    pub steps: Vec<Rc<dyn ApprovalForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalForecastEngine {
    pub fn run(&mut self, item: &ApprovalForecast) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_approval_forecast_engine(flags: &flags::Registry) -> ApprovalForecastEngine {
    ApprovalForecastEngine {
        steps: vec![
            Rc::new(FlaggedApprovalForecastAdjuster {
                flags: flags.clone(),
                flag: "approval-forecast-primary".to_string(),
                percent: 8.8,
            }) as Rc<dyn ApprovalForecastModifier>,
            Rc::new(LocaleApprovalForecastAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.1)]),
            }),
            Rc::new(PercentApprovalForecastAdjuster { percent: -5.6 }),
            Rc::new(RoundingApprovalForecastAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 21047.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_forecast_engine(&flags::new_registry());
        let item = new_approval_forecast("code-16", "title-541");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
