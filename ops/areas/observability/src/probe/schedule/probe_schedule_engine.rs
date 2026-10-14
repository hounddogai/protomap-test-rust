use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the probe schedule calculation.
pub trait ProbeScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelProbeScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ProbeScheduleFactor for LabelProbeScheduleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ProbeSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbeScheduleAdjuster {
    pub percent: f64,
}

impl ProbeScheduleFactor for PercentProbeScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbeSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbeScheduleFactor for TieredProbeScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbeSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedProbeScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ProbeScheduleFactor for FlaggedProbeScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ProbeSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the probe schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeScheduleEngine {
    pub steps: Vec<Rc<dyn ProbeScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeScheduleEngine {
    pub fn compute(&mut self, item: &ProbeSchedule) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ProbeScheduleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ProbeSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_probe_schedule_engine(flags: &flags::Registry) -> ProbeScheduleEngine {
    ProbeScheduleEngine {
        steps: vec![
            Rc::new(LabelProbeScheduleAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.26)]),
            }) as Rc<dyn ProbeScheduleFactor>,
            Rc::new(PercentProbeScheduleAdjuster { percent: 7.9 }),
            Rc::new(TieredProbeScheduleAdjuster {
                bounds: vec![495.0, 751.0, 7097.0, 7843.0],
                rates: vec![1.2, 1.01, 0.99, 0.91],
            }),
            Rc::new(FlaggedProbeScheduleAdjuster {
                flags: flags.clone(),
                flag: "probe-schedule-fallback".to_string(),
                percent: -8.9,
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 94374.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_schedule_engine(&flags::new_registry());
        let item = new_probe_schedule("code-872", "label-40");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
