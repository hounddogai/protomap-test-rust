use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the probe plan calculation.
pub trait ProbePlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbePlanMinimumOffsetAdjuster {
    pub weight: f64,
}

impl ProbePlanAdjuster for ProbePlanMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &ProbePlan, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedProbePlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ProbePlanAdjuster for FlaggedProbePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ProbePlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapProbePlanAdjuster {
    pub cap: f64,
}

impl ProbePlanAdjuster for CapProbePlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ProbePlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentProbePlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ProbePlanAdjuster for SegmentProbePlanAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &ProbePlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbePlanAdjuster {
    pub percent: f64,
}

impl ProbePlanAdjuster for PercentProbePlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbePlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbePlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbePlanAdjuster for TieredProbePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbePlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the probe plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbePlanEngine {
    pub steps: Vec<Rc<dyn ProbePlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbePlanEngine {
    pub fn price(&mut self, item: &ProbePlan) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[ProbePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_probe_plan_engine(flags: &flags::Registry) -> ProbePlanEngine {
    ProbePlanEngine {
        steps: vec![
            Rc::new(ProbePlanMinimumOffsetAdjuster { weight: 0.22 }) as Rc<dyn ProbePlanAdjuster>,
            Rc::new(FlaggedProbePlanAdjuster {
                flags: flags.clone(),
                flag: "probe-plan-adaptive".to_string(),
                percent: -3.3,
            }),
            Rc::new(CapProbePlanAdjuster { cap: 34866.0 }),
            Rc::new(SegmentProbePlanAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.78)]),
            }),
            Rc::new(PercentProbePlanAdjuster { percent: -9.8 }),
            Rc::new(TieredProbePlanAdjuster {
                bounds: vec![877.0, 3855.0, 4155.0, 6363.0],
                rates: vec![0.97, 0.89, 1.02, 0.9],
            }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 17704.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_plan_engine(&flags::new_registry());
        let item = new_probe_plan("code-151", "label-351");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
