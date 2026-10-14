use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the migration forecast calculation.
pub trait MigrationForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationForecastAdjuster {
    pub percent: f64,
}

impl MigrationForecastStep for PercentMigrationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedMigrationForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MigrationForecastStep for FlaggedMigrationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MigrationForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceMigrationForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MigrationForecastStep for SourceMigrationForecastAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &MigrationForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the migration forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationForecastEngine {
    pub steps: Vec<Rc<dyn MigrationForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationForecastEngine {
    pub fn run(&mut self, item: &MigrationForecast) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationForecastStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_migration_forecast_engine(flags: &flags::Registry) -> MigrationForecastEngine {
    MigrationForecastEngine {
        steps: vec![
            Rc::new(PercentMigrationForecastAdjuster { percent: -5.5 }) as Rc<dyn MigrationForecastStep>,
            Rc::new(FlaggedMigrationForecastAdjuster {
                flags: flags.clone(),
                flag: "migration-forecast-strict".to_string(),
                percent: 1.6,
            }),
            Rc::new(SourceMigrationForecastAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.78)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 73740.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_forecast_engine(&flags::new_registry());
        let item = new_migration_forecast("code-619", "name-737");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
