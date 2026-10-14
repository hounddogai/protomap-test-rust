use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient probe group calculation.
pub trait LenientProbeGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientProbeGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLenientProbeGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientProbeGroupStep for TitleLenientProbeGroupAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &LenientProbeGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientProbeGroupAdjuster {
    pub step: f64,
}

impl LenientProbeGroupStep for RoundingLenientProbeGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientProbeGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientProbeGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientProbeGroupStep for TieredLenientProbeGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientProbeGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientProbeGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientProbeGroupStep for FlaggedLenientProbeGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientProbeGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeGroupRatioOffsetAdjuster {
    pub weight: f64,
}

impl LenientProbeGroupStep for LenientProbeGroupRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &LenientProbeGroup, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientProbeGroupAdjuster {
    pub minimum: f64,
}

impl LenientProbeGroupStep for MinimumLenientProbeGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientProbeGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient probe group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientProbeGroupEngine {
    pub steps: Vec<Rc<dyn LenientProbeGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientProbeGroupEngine {
    pub fn run(&mut self, item: &LenientProbeGroup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LenientProbeGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_probe_group_engine(flags: &flags::Registry) -> LenientProbeGroupEngine {
    LenientProbeGroupEngine {
        steps: vec![
            Rc::new(TitleLenientProbeGroupAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.38), ("west".to_string(), 0.74)]),
            }) as Rc<dyn LenientProbeGroupStep>,
            Rc::new(RoundingLenientProbeGroupAdjuster { step: 0.25 }),
            Rc::new(TieredLenientProbeGroupAdjuster {
                bounds: vec![1460.0, 3418.0, 4478.0, 9350.0],
                rates: vec![1.2, 1.2, 0.96, 1.15],
            }),
            Rc::new(FlaggedLenientProbeGroupAdjuster {
                flags: flags.clone(),
                flag: "lenient-probe-group-global".to_string(),
                percent: -5.6,
            }),
            Rc::new(LenientProbeGroupRatioOffsetAdjuster { weight: 1.26 }),
            Rc::new(MinimumLenientProbeGroupAdjuster { minimum: 20.9 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 1078.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_probe_group_engine(&flags::new_registry());
        let item = new_lenient_probe_group("code-324", "title-161");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
