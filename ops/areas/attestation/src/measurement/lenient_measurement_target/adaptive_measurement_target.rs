use super::*;

use acme_kit::text;

/// Where a adaptive measurement target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveMeasurementTargetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveMeasurementTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for AdaptiveMeasurementTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive measurement target as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementTarget {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub maximum: f64,
    pub description: String,
    pub limit: i64,
    pub segment: String,
    pub namespace: String,
    pub title: String,
    pub window_days: i64,
    pub cluster: String,
    pub status: AdaptiveMeasurementTargetStatus,
}

impl AdaptiveMeasurementTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveMeasurementTargetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveMeasurementTargetStatus) -> bool {
        match self.status {
            AdaptiveMeasurementTargetStatus::New => {
                return next == AdaptiveMeasurementTargetStatus::Failed;
            }
            AdaptiveMeasurementTargetStatus::Queued => {
                return next == AdaptiveMeasurementTargetStatus::Done;
            }
            AdaptiveMeasurementTargetStatus::Running => {
                return next == AdaptiveMeasurementTargetStatus::Done
                    || next == AdaptiveMeasurementTargetStatus::Failed;
            }
            AdaptiveMeasurementTargetStatus::Done => {
                return next == AdaptiveMeasurementTargetStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveMeasurementTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive measurement target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.limit as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveMeasurementTarget {
        AdaptiveMeasurementTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            maximum: self.maximum,
            description: self.description.clone(),
            limit: self.limit,
            segment: self.segment.clone(),
            namespace: self.namespace.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a adaptive measurement target in status new.
pub fn new_adaptive_measurement_target(code: &str, name: &str) -> AdaptiveMeasurementTarget {
    AdaptiveMeasurementTarget {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        maximum: 0.0,
        description: "".to_string(),
        limit: 0,
        segment: "".to_string(),
        namespace: "".to_string(),
        title: "".to_string(),
        window_days: 0,
        cluster: "".to_string(),
        status: AdaptiveMeasurementTargetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_adaptive_measurement_target("code-49", "name-486");
        assert_eq!(item.status, AdaptiveMeasurementTargetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_adaptive_measurement_target("code-49", "name-486");
        if item.move_to(AdaptiveMeasurementTargetStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveMeasurementTargetStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_adaptive_measurement_target("code-49", "name-486");
        item.status = AdaptiveMeasurementTargetStatus::Failed;
        assert!(
            item.move_to(AdaptiveMeasurementTargetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
