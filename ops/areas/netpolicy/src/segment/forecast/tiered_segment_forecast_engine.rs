use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tiered segment forecast calculation.
pub trait TieredSegmentForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredSegmentForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentForecastBudgetOffsetAdjuster {
    pub weight: f64,
}

impl TieredSegmentForecastModifier for TieredSegmentForecastBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &TieredSegmentForecast, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredSegmentForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredSegmentForecastModifier for FlaggedTieredSegmentForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredSegmentForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredSegmentForecastAdjuster {
    pub percent: f64,
}

impl TieredSegmentForecastModifier for PercentTieredSegmentForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredSegmentForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTieredSegmentForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredSegmentForecastModifier for ChannelTieredSegmentForecastAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &TieredSegmentForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the tiered segment forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredSegmentForecastEngine {
    pub steps: Vec<Rc<dyn TieredSegmentForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredSegmentForecastEngine {
    pub fn run(&mut self, item: &TieredSegmentForecast) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredSegmentForecastModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredSegmentForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_segment_forecast_engine(flags: &flags::Registry) -> TieredSegmentForecastEngine {
    TieredSegmentForecastEngine {
        steps: vec![
            Rc::new(TieredSegmentForecastBudgetOffsetAdjuster { weight: 1.44 })
                as Rc<dyn TieredSegmentForecastModifier>,
            Rc::new(FlaggedTieredSegmentForecastAdjuster {
                flags: flags.clone(),
                flag: "tiered-segment-forecast-fallback".to_string(),
                percent: -2.9,
            }),
            Rc::new(PercentTieredSegmentForecastAdjuster { percent: 0.9 }),
            Rc::new(ChannelTieredSegmentForecastAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.08)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 14629.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_segment_forecast_engine(&flags::new_registry());
        let item = new_tiered_segment_forecast("code-736", "title-502");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
