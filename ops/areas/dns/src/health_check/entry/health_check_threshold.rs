use super::*;

use acme_kit::text;

/// Where a health check threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckThresholdStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl HealthCheckThresholdStatus {
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

impl std::fmt::Display for HealthCheckThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckThreshold {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub archived: bool,
    pub tags: Vec<String>,
    pub active: bool,
    pub unit: String,
    pub status: HealthCheckThresholdStatus,
}

impl HealthCheckThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckThresholdStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckThresholdStatus) -> bool {
        match self.status {
            HealthCheckThresholdStatus::New => {
                match next {
                    HealthCheckThresholdStatus::Running | HealthCheckThresholdStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckThresholdStatus::Queued => {
                return next == HealthCheckThresholdStatus::Running;
            }
            HealthCheckThresholdStatus::Running => {
                match next {
                    HealthCheckThresholdStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckThresholdStatus::Done => {
                return next == HealthCheckThresholdStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.rate
    }

    pub fn copy(&self) -> HealthCheckThreshold {
        HealthCheckThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            archived: self.archived,
            tags: self.tags.clone(),
            active: self.active,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a health check threshold in status new.
pub fn new_health_check_threshold(code: &str, title: &str) -> HealthCheckThreshold {
    HealthCheckThreshold {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        archived: false,
        tags: Vec::new(),
        active: false,
        unit: "".to_string(),
        status: HealthCheckThresholdStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_health_check_threshold("code-579", "title-319");
        assert_eq!(item.status, HealthCheckThresholdStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_health_check_threshold("code-579", "title-319");
        if item.move_to(HealthCheckThresholdStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckThresholdStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_health_check_threshold("code-579", "title-319");
        item.status = HealthCheckThresholdStatus::Failed;
        assert!(
            item.move_to(HealthCheckThresholdStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
