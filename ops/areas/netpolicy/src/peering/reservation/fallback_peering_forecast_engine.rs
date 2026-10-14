use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback peering forecast calculation.
pub trait FallbackPeeringForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackPeeringForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelFallbackPeeringForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackPeeringForecastFactor for ChannelFallbackPeeringForecastAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &FallbackPeeringForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackPeeringForecastAdjuster {
    pub step: f64,
}

impl FallbackPeeringForecastFactor for RoundingFallbackPeeringForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackPeeringForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackPeeringForecastAdjuster {
    pub cap: f64,
}

impl FallbackPeeringForecastFactor for CapFallbackPeeringForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackPeeringForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackPeeringForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackPeeringForecastFactor for TieredFallbackPeeringForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackPeeringForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackPeeringForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackPeeringForecastFactor for FlaggedFallbackPeeringForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackPeeringForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback peering forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackPeeringForecastEngine {
    pub steps: Vec<Rc<dyn FallbackPeeringForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackPeeringForecastEngine {
    pub fn compute(&mut self, item: &FallbackPeeringForecast) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[FallbackPeeringForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_peering_forecast_engine(flags: &flags::Registry) -> FallbackPeeringForecastEngine {
    FallbackPeeringForecastEngine {
        steps: vec![
            Rc::new(ChannelFallbackPeeringForecastAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.84)]),
            }) as Rc<dyn FallbackPeeringForecastFactor>,
            Rc::new(RoundingFallbackPeeringForecastAdjuster { step: 1.0 }),
            Rc::new(CapFallbackPeeringForecastAdjuster { cap: 45949.0 }),
            Rc::new(TieredFallbackPeeringForecastAdjuster {
                bounds: vec![3188.0, 7252.0, 7727.0, 9969.0],
                rates: vec![1.01, 0.92, 0.89, 0.96],
            }),
            Rc::new(FlaggedFallbackPeeringForecastAdjuster {
                flags: flags.clone(),
                flag: "fallback-peering-forecast-adaptive".to_string(),
                percent: -7.9,
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 15665.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_peering_forecast_engine(&flags::new_registry());
        let item = new_fallback_peering_forecast("code-987", "name-237");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
