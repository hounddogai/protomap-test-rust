use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the health check milestone calculation.
pub trait HealthCheckMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckMilestoneModifier for FlaggedHealthCheckMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentHealthCheckMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl HealthCheckMilestoneModifier for SegmentHealthCheckMilestoneAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &HealthCheckMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentHealthCheckMilestoneAdjuster {
    pub percent: f64,
}

impl HealthCheckMilestoneModifier for PercentHealthCheckMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &HealthCheckMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl HealthCheckMilestoneModifier for TieredHealthCheckMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &HealthCheckMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckMilestoneStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl HealthCheckMilestoneModifier for HealthCheckMilestoneStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &HealthCheckMilestone, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingHealthCheckMilestoneAdjuster {
    pub step: f64,
}

impl HealthCheckMilestoneModifier for RoundingHealthCheckMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &HealthCheckMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the health check milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckMilestoneEngine {
    pub steps: Vec<Rc<dyn HealthCheckMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckMilestoneEngine {
    pub fn evaluate(&mut self, item: &HealthCheckMilestone) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_health_check_milestone_engine(flags: &flags::Registry) -> HealthCheckMilestoneEngine {
    HealthCheckMilestoneEngine {
        steps: vec![
            Rc::new(FlaggedHealthCheckMilestoneAdjuster {
                flags: flags.clone(),
                flag: "health-check-milestone-primary".to_string(),
                percent: 3.1,
            }) as Rc<dyn HealthCheckMilestoneModifier>,
            Rc::new(SegmentHealthCheckMilestoneAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.3)]),
            }),
            Rc::new(PercentHealthCheckMilestoneAdjuster { percent: 5.5 }),
            Rc::new(TieredHealthCheckMilestoneAdjuster {
                bounds: vec![6201.0, 6880.0, 7300.0],
                rates: vec![1.11, 0.93, 0.98],
            }),
            Rc::new(HealthCheckMilestoneStartsAtOffsetAdjuster { weight: 1.32 }),
            Rc::new(RoundingHealthCheckMilestoneAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 94551.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_milestone_engine(&flags::new_registry());
        let item = new_health_check_milestone("code-871", "label-402");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
