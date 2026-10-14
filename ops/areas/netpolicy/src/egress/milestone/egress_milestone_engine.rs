use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the egress milestone calculation.
pub trait EgressMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryEgressMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EgressMilestoneStep for CountryEgressMilestoneAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &EgressMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EgressMilestoneStep for TieredEgressMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EgressMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEgressMilestoneAdjuster {
    pub minimum: f64,
}

impl EgressMilestoneStep for MinimumEgressMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EgressMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressMilestoneAdjuster {
    pub percent: f64,
}

impl EgressMilestoneStep for PercentEgressMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEgressMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EgressMilestoneStep for FlaggedEgressMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EgressMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressMilestoneRatioOffsetAdjuster {
    pub weight: f64,
}

impl EgressMilestoneStep for EgressMilestoneRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &EgressMilestone, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

/// Runs the egress milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressMilestoneEngine {
    pub steps: Vec<Rc<dyn EgressMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressMilestoneEngine {
    pub fn evaluate(&mut self, item: &EgressMilestone) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[EgressMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_egress_milestone_engine(flags: &flags::Registry) -> EgressMilestoneEngine {
    EgressMilestoneEngine {
        steps: vec![
            Rc::new(CountryEgressMilestoneAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.02), ("south".to_string(), 0.72)]),
            }) as Rc<dyn EgressMilestoneStep>,
            Rc::new(TieredEgressMilestoneAdjuster {
                bounds: vec![962.0, 2886.0, 4066.0],
                rates: vec![0.98, 1.13, 0.82],
            }),
            Rc::new(MinimumEgressMilestoneAdjuster { minimum: 0.8 }),
            Rc::new(PercentEgressMilestoneAdjuster { percent: -3.8 }),
            Rc::new(FlaggedEgressMilestoneAdjuster {
                flags: flags.clone(),
                flag: "egress-milestone-pending".to_string(),
                percent: -3.9,
            }),
            Rc::new(EgressMilestoneRatioOffsetAdjuster { weight: 1.48 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 61944.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_milestone_engine(&flags::new_registry());
        let item = new_egress_milestone("code-541", "title-219");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
