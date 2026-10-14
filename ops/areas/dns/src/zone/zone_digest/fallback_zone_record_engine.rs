use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback zone record calculation.
pub trait FallbackZoneRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackZoneRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackZoneRecordAdjuster {
    pub percent: f64,
}

impl FallbackZoneRecordAdjuster for PercentFallbackZoneRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackZoneRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackZoneRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackZoneRecordAdjuster for TieredFallbackZoneRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackZoneRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackZoneRecordAdjuster {
    pub minimum: f64,
}

impl FallbackZoneRecordAdjuster for MinimumFallbackZoneRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackZoneRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackZoneRecordAdjuster {
    pub cap: f64,
}

impl FallbackZoneRecordAdjuster for CapFallbackZoneRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackZoneRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneRecordScoreOffsetAdjuster {
    pub weight: f64,
}

impl FallbackZoneRecordAdjuster for FallbackZoneRecordScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackZoneRecord, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFallbackZoneRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackZoneRecordAdjuster for TitleFallbackZoneRecordAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FallbackZoneRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback zone record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackZoneRecordEngine {
    pub steps: Vec<Rc<dyn FallbackZoneRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackZoneRecordEngine {
    pub fn price(&mut self, item: &FallbackZoneRecord) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FallbackZoneRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_zone_record_engine() -> FallbackZoneRecordEngine {
    FallbackZoneRecordEngine {
        steps: vec![
            Rc::new(PercentFallbackZoneRecordAdjuster { percent: 13.3 }) as Rc<dyn FallbackZoneRecordAdjuster>,
            Rc::new(TieredFallbackZoneRecordAdjuster {
                bounds: vec![431.0, 5231.0],
                rates: vec![1.19, 0.89],
            }),
            Rc::new(MinimumFallbackZoneRecordAdjuster { minimum: 34.4 }),
            Rc::new(CapFallbackZoneRecordAdjuster { cap: 22145.0 }),
            Rc::new(FallbackZoneRecordScoreOffsetAdjuster { weight: 0.32 }),
            Rc::new(TitleFallbackZoneRecordAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.91),
                    ("online".to_string(), 0.74),
                    ("central".to_string(), 0.96),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 92757.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_zone_record_engine();
        let item = new_fallback_zone_record("code-553", "title-485");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
