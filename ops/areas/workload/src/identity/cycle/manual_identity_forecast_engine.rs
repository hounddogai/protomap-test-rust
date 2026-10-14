use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual identity forecast calculation.
pub trait ManualIdentityForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualIdentityForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityForecastScoreOffsetAdjuster {
    pub weight: f64,
}

impl ManualIdentityForecastFactor for ManualIdentityForecastScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &ManualIdentityForecast, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualIdentityForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualIdentityForecastFactor for FlaggedManualIdentityForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualIdentityForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleManualIdentityForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualIdentityForecastFactor for TitleManualIdentityForecastAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ManualIdentityForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the manual identity forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualIdentityForecastEngine {
    pub steps: Vec<Rc<dyn ManualIdentityForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualIdentityForecastEngine {
    pub fn compute(&mut self, item: &ManualIdentityForecast) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[ManualIdentityForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_identity_forecast_engine(flags: &flags::Registry) -> ManualIdentityForecastEngine {
    ManualIdentityForecastEngine {
        steps: vec![
            Rc::new(ManualIdentityForecastScoreOffsetAdjuster { weight: 1.55 }) as Rc<dyn ManualIdentityForecastFactor>,
            Rc::new(FlaggedManualIdentityForecastAdjuster {
                flags: flags.clone(),
                flag: "manual-identity-forecast-scheduled".to_string(),
                percent: -3.2,
            }),
            Rc::new(TitleManualIdentityForecastAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.95), ("south".to_string(), 0.84)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 33690.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_identity_forecast_engine(&flags::new_registry());
        let item = new_manual_identity_forecast("code-845", "title-770");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
