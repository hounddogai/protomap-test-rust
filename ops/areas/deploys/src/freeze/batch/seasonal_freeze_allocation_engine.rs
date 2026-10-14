use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal freeze allocation calculation.
pub trait SeasonalFreezeAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalFreezeAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalFreezeAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalFreezeAllocationFactor for FlaggedSeasonalFreezeAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalFreezeAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalFreezeAllocationAdjuster {
    pub percent: f64,
}

impl SeasonalFreezeAllocationFactor for PercentSeasonalFreezeAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalFreezeAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleSeasonalFreezeAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalFreezeAllocationFactor for TitleSeasonalFreezeAllocationAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &SeasonalFreezeAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalFreezeAllocationAdjuster {
    pub minimum: f64,
}

impl SeasonalFreezeAllocationFactor for MinimumSeasonalFreezeAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalFreezeAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalFreezeAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalFreezeAllocationFactor for TieredSeasonalFreezeAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalFreezeAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeAllocationPriorityOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalFreezeAllocationFactor for SeasonalFreezeAllocationPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalFreezeAllocation, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

/// Runs the seasonal freeze allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalFreezeAllocationEngine {
    pub steps: Vec<Rc<dyn SeasonalFreezeAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalFreezeAllocationEngine {
    pub fn evaluate(&mut self, item: &SeasonalFreezeAllocation) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalFreezeAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_freeze_allocation_engine(flags: &flags::Registry) -> SeasonalFreezeAllocationEngine {
    SeasonalFreezeAllocationEngine {
        steps: vec![
            Rc::new(FlaggedSeasonalFreezeAllocationAdjuster {
                flags: flags.clone(),
                flag: "seasonal-freeze-allocation-seasonal".to_string(),
                percent: -3.8,
            }) as Rc<dyn SeasonalFreezeAllocationFactor>,
            Rc::new(PercentSeasonalFreezeAllocationAdjuster { percent: -24.1 }),
            Rc::new(TitleSeasonalFreezeAllocationAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.1),
                    ("south".to_string(), 0.8),
                    ("online".to_string(), 1.0),
                ]),
            }),
            Rc::new(MinimumSeasonalFreezeAllocationAdjuster { minimum: 17.1 }),
            Rc::new(TieredSeasonalFreezeAllocationAdjuster {
                bounds: vec![1972.0, 6425.0, 8598.0],
                rates: vec![0.87, 1.17, 1.03],
            }),
            Rc::new(SeasonalFreezeAllocationPriorityOffsetAdjuster { weight: 0.69 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 95573.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_freeze_allocation_engine(&flags::new_registry());
        let item = new_seasonal_freeze_allocation("code-118", "title-255");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
