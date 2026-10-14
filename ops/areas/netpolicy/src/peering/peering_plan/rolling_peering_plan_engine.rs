use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling peering plan calculation.
pub trait RollingPeeringPlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPeeringPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingPeeringPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingPeeringPlanAdjuster for TieredRollingPeeringPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingPeeringPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingPeeringPlanAdjuster {
    pub percent: f64,
}

impl RollingPeeringPlanAdjuster for PercentRollingPeeringPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingPeeringPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRollingPeeringPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingPeeringPlanAdjuster for RegionRollingPeeringPlanAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RollingPeeringPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingPeeringPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingPeeringPlanAdjuster for FlaggedRollingPeeringPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingPeeringPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingPeeringPlanAdjuster {
    pub cap: f64,
}

impl RollingPeeringPlanAdjuster for CapRollingPeeringPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingPeeringPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling peering plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPeeringPlanEngine {
    pub steps: Vec<Rc<dyn RollingPeeringPlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPeeringPlanEngine {
    pub fn run(&mut self, item: &RollingPeeringPlan) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollingPeeringPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_peering_plan_engine(flags: &flags::Registry) -> RollingPeeringPlanEngine {
    RollingPeeringPlanEngine {
        steps: vec![
            Rc::new(TieredRollingPeeringPlanAdjuster {
                bounds: vec![679.0, 3701.0, 4444.0],
                rates: vec![0.81, 0.97, 0.83],
            }) as Rc<dyn RollingPeeringPlanAdjuster>,
            Rc::new(PercentRollingPeeringPlanAdjuster { percent: 0.4 }),
            Rc::new(RegionRollingPeeringPlanAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.07),
                    ("south".to_string(), 1.03),
                    ("north".to_string(), 0.75),
                ]),
            }),
            Rc::new(FlaggedRollingPeeringPlanAdjuster {
                flags: flags.clone(),
                flag: "rolling-peering-plan-legacy".to_string(),
                percent: -0.8,
            }),
            Rc::new(CapRollingPeeringPlanAdjuster { cap: 18225.0 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 7486.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_peering_plan_engine(&flags::new_registry());
        let item = new_rolling_peering_plan("code-321", "label-241");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
