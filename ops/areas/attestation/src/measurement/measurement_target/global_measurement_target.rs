use super::*;

use acme_kit::text;

/// Where a global measurement target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalMeasurementTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalMeasurementTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalMeasurementTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global measurement target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementTarget {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub issuer: String,
    pub ceiling: f64,
    pub window_days: i64,
    pub unit: String,
    pub limit: i64,
    pub status: GlobalMeasurementTargetStatus,
}

impl GlobalMeasurementTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalMeasurementTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalMeasurementTargetStatus) -> bool {
        match self.status {
            GlobalMeasurementTargetStatus::Created => {
                match next {
                    GlobalMeasurementTargetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalMeasurementTargetStatus::Validated => {
                return next == GlobalMeasurementTargetStatus::Published;
            }
            GlobalMeasurementTargetStatus::Published => {
                match next {
                    GlobalMeasurementTargetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalMeasurementTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global measurement target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.ceiling + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalMeasurementTarget {
        GlobalMeasurementTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            issuer: self.issuer.clone(),
            ceiling: self.ceiling,
            window_days: self.window_days,
            unit: self.unit.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a global measurement target in status created.
pub fn new_global_measurement_target(code: &str, title: &str) -> GlobalMeasurementTarget {
    GlobalMeasurementTarget {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        issuer: "".to_string(),
        ceiling: 0.0,
        window_days: 0,
        unit: "".to_string(),
        limit: 0,
        status: GlobalMeasurementTargetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_global_measurement_target("code-703", "title-931");
        assert_eq!(item.status, GlobalMeasurementTargetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_global_measurement_target("code-703", "title-931");
        if item.move_to(GlobalMeasurementTargetStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalMeasurementTargetStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_global_measurement_target("code-703", "title-931");
        item.status = GlobalMeasurementTargetStatus::Retired;
        assert!(
            item.move_to(GlobalMeasurementTargetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
