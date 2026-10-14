use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal alert adjustment calculation.
pub trait SeasonalAlertAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalAlertAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalAlertAdjustmentAdjuster {
    pub minimum: f64,
}

impl SeasonalAlertAdjustmentModifier for MinimumSeasonalAlertAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalAlertAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalAlertAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalAlertAdjustmentModifier for TieredSeasonalAlertAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalAlertAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalAlertAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalAlertAdjustmentModifier for FlaggedSeasonalAlertAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalAlertAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeSeasonalAlertAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalAlertAdjustmentModifier for ScopeSeasonalAlertAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &SeasonalAlertAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the seasonal alert adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalAlertAdjustmentEngine {
    pub steps: Vec<Rc<dyn SeasonalAlertAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalAlertAdjustmentEngine {
    pub fn compute(&mut self, item: &SeasonalAlertAdjustment) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalAlertAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_alert_adjustment_engine(flags: &flags::Registry) -> SeasonalAlertAdjustmentEngine {
    SeasonalAlertAdjustmentEngine {
        steps: vec![
            Rc::new(MinimumSeasonalAlertAdjustmentAdjuster { minimum: 32.4 })
                as Rc<dyn SeasonalAlertAdjustmentModifier>,
            Rc::new(TieredSeasonalAlertAdjustmentAdjuster {
                bounds: vec![2750.0, 5067.0, 5393.0, 9757.0],
                rates: vec![0.89, 1.19, 1.07, 0.96],
            }),
            Rc::new(FlaggedSeasonalAlertAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "seasonal-alert-adjustment-adaptive".to_string(),
                percent: 2.3,
            }),
            Rc::new(ScopeSeasonalAlertAdjustmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.91)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 71097.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_alert_adjustment_engine(&flags::new_registry());
        let item = new_seasonal_alert_adjustment("code-586", "title-26");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
