use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback region pin summary calculation.
pub trait FallbackRegionPinSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRegionPinSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackRegionPinSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackRegionPinSummaryModifier for TieredFallbackRegionPinSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackRegionPinSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackRegionPinSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackRegionPinSummaryModifier for FlaggedFallbackRegionPinSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackRegionPinSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryFallbackRegionPinSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackRegionPinSummaryModifier for CountryFallbackRegionPinSummaryAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &FallbackRegionPinSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback region pin summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRegionPinSummaryEngine {
    pub steps: Vec<Rc<dyn FallbackRegionPinSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRegionPinSummaryEngine {
    pub fn price(&mut self, item: &FallbackRegionPinSummary) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FallbackRegionPinSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_region_pin_summary_engine(flags: &flags::Registry) -> FallbackRegionPinSummaryEngine {
    FallbackRegionPinSummaryEngine {
        steps: vec![
            Rc::new(TieredFallbackRegionPinSummaryAdjuster {
                bounds: vec![2165.0, 2897.0, 3402.0, 5982.0],
                rates: vec![1.09, 0.81, 1.05, 1.03],
            }) as Rc<dyn FallbackRegionPinSummaryModifier>,
            Rc::new(FlaggedFallbackRegionPinSummaryAdjuster {
                flags: flags.clone(),
                flag: "fallback-region-pin-summary-strict".to_string(),
                percent: 9.2,
            }),
            Rc::new(CountryFallbackRegionPinSummaryAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.1)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 24977.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_region_pin_summary_engine(&flags::new_registry());
        let item = new_fallback_region_pin_summary("code-301", "title-659");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
