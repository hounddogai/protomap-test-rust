use super::*;

use acme_kit::text;

/// Where a health check checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl HealthCheckCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for HealthCheckCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check checkpoint as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckCheckpoint {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub rate: f64,
    pub attempts: i64,
    pub ttl_seconds: i64,
    pub category: String,
    pub minimum: f64,
    pub source: String,
    pub key_id: String,
    pub revision: i64,
    pub status: HealthCheckCheckpointStatus,
}

impl HealthCheckCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckCheckpointStatus) -> bool {
        match self.status {
            HealthCheckCheckpointStatus::Draft => {
                return next == HealthCheckCheckpointStatus::Active || next == HealthCheckCheckpointStatus::Closed;
            }
            HealthCheckCheckpointStatus::Pending => {
                return next == HealthCheckCheckpointStatus::Closed;
            }
            HealthCheckCheckpointStatus::Active => {
                return next == HealthCheckCheckpointStatus::Suspended || next == HealthCheckCheckpointStatus::Closed;
            }
            HealthCheckCheckpointStatus::Suspended => {
                match next {
                    HealthCheckCheckpointStatus::Closed => {
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

    pub fn move_to(&mut self, next: HealthCheckCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "health check checkpoint", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.minimum - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckCheckpoint {
        HealthCheckCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            rate: self.rate,
            attempts: self.attempts,
            ttl_seconds: self.ttl_seconds,
            category: self.category.clone(),
            minimum: self.minimum,
            source: self.source.clone(),
            key_id: self.key_id.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a health check checkpoint in status draft.
pub fn new_health_check_checkpoint(code: &str, label: &str) -> HealthCheckCheckpoint {
    HealthCheckCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        rate: 0.0,
        attempts: 0,
        ttl_seconds: 0,
        category: "".to_string(),
        minimum: 0.0,
        source: "".to_string(),
        key_id: "".to_string(),
        revision: 0,
        status: HealthCheckCheckpointStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_health_check_checkpoint("code-807", "label-838");
        assert_eq!(item.status, HealthCheckCheckpointStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_health_check_checkpoint("code-807", "label-838");
        if item.move_to(HealthCheckCheckpointStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckCheckpointStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_health_check_checkpoint("code-807", "label-838");
        item.status = HealthCheckCheckpointStatus::Closed;
        assert!(
            item.move_to(HealthCheckCheckpointStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
