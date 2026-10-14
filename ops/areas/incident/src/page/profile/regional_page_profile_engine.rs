use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional page profile calculation.
pub trait RegionalPageProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalPageProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalPageProfileAdjuster {
    pub minimum: f64,
}

impl RegionalPageProfileModifier for MinimumRegionalPageProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalPageProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalPageProfileAdjuster {
    pub percent: f64,
}

impl RegionalPageProfileModifier for PercentRegionalPageProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalPageProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalPageProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalPageProfileModifier for FlaggedRegionalPageProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalPageProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalPageProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalPageProfileModifier for TieredRegionalPageProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalPageProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the regional page profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalPageProfileEngine {
    pub steps: Vec<Rc<dyn RegionalPageProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalPageProfileEngine {
    pub fn run(&mut self, item: &RegionalPageProfile) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[RegionalPageProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_page_profile_engine(flags: &flags::Registry) -> RegionalPageProfileEngine {
    RegionalPageProfileEngine {
        steps: vec![
            Rc::new(MinimumRegionalPageProfileAdjuster { minimum: 15.5 }) as Rc<dyn RegionalPageProfileModifier>,
            Rc::new(PercentRegionalPageProfileAdjuster { percent: 23.8 }),
            Rc::new(FlaggedRegionalPageProfileAdjuster {
                flags: flags.clone(),
                flag: "regional-page-profile-batched".to_string(),
                percent: -6.0,
            }),
            Rc::new(TieredRegionalPageProfileAdjuster {
                bounds: vec![657.0, 2782.0, 3029.0, 7316.0],
                rates: vec![0.82, 1.09, 0.84, 1.17],
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 91391.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_page_profile_engine(&flags::new_registry());
        let item = new_regional_page_profile("code-439", "title-309");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
