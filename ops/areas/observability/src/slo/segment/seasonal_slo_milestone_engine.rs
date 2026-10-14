use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal slo milestone calculation.
pub trait SeasonalSloMilestoneAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalSloMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalSloMilestoneAdjuster {
    pub step: f64,
}

impl SeasonalSloMilestoneAdjuster for RoundingSeasonalSloMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalSloMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalSloMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalSloMilestoneAdjuster for FlaggedSeasonalSloMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalSloMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalSloMilestoneAdjuster {
    pub percent: f64,
}

impl SeasonalSloMilestoneAdjuster for PercentSeasonalSloMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalSloMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitSeasonalSloMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalSloMilestoneAdjuster for UnitSeasonalSloMilestoneAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &SeasonalSloMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalSloMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalSloMilestoneAdjuster for TieredSeasonalSloMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalSloMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the seasonal slo milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalSloMilestoneEngine {
    pub steps: Vec<Rc<dyn SeasonalSloMilestoneAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalSloMilestoneEngine {
    pub fn evaluate(&mut self, item: &SeasonalSloMilestone) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalSloMilestoneAdjuster> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[SeasonalSloMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_slo_milestone_engine(flags: &flags::Registry) -> SeasonalSloMilestoneEngine {
    SeasonalSloMilestoneEngine {
        steps: vec![
            Rc::new(RoundingSeasonalSloMilestoneAdjuster { step: 0.25 }) as Rc<dyn SeasonalSloMilestoneAdjuster>,
            Rc::new(FlaggedSeasonalSloMilestoneAdjuster {
                flags: flags.clone(),
                flag: "seasonal-slo-milestone-regional".to_string(),
                percent: 2.2,
            }),
            Rc::new(PercentSeasonalSloMilestoneAdjuster { percent: -3.3 }),
            Rc::new(UnitSeasonalSloMilestoneAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.32),
                    ("online".to_string(), 0.82),
                    ("retail".to_string(), 1.2),
                ]),
            }),
            Rc::new(TieredSeasonalSloMilestoneAdjuster {
                bounds: vec![764.0, 5286.0, 6994.0, 7939.0],
                rates: vec![1.04, 1.19, 0.81, 1.13],
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 75641.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_slo_milestone_engine(&flags::new_registry());
        let item = new_seasonal_slo_milestone("code-447", "title-937");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
