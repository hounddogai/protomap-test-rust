use super::*;

use acme_kit::text;

/// Where a batched measurement quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedMeasurementQuotaStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedMeasurementQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for BatchedMeasurementQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched measurement quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMeasurementQuota {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub ceiling: f64,
    pub timezone: String,
    pub duration_minutes: i64,
    pub port: i64,
    pub created_at: i64,
    pub source: String,
    pub region: String,
    pub status: BatchedMeasurementQuotaStatus,
}

impl BatchedMeasurementQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedMeasurementQuotaStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedMeasurementQuotaStatus) -> bool {
        match self.status {
            BatchedMeasurementQuotaStatus::Planned => {
                match next {
                    BatchedMeasurementQuotaStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedMeasurementQuotaStatus::Scheduled => {
                return next == BatchedMeasurementQuotaStatus::InProgress
                    || next == BatchedMeasurementQuotaStatus::Cancelled;
            }
            BatchedMeasurementQuotaStatus::InProgress => {
                match next {
                    BatchedMeasurementQuotaStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedMeasurementQuotaStatus::Completed => {
                match next {
                    BatchedMeasurementQuotaStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: BatchedMeasurementQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched measurement quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.created_at as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedMeasurementQuota {
        BatchedMeasurementQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            ceiling: self.ceiling,
            timezone: self.timezone.clone(),
            duration_minutes: self.duration_minutes,
            port: self.port,
            created_at: self.created_at,
            source: self.source.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a batched measurement quota in status planned.
pub fn new_batched_measurement_quota(code: &str, label: &str) -> BatchedMeasurementQuota {
    BatchedMeasurementQuota {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        ceiling: 0.0,
        timezone: "".to_string(),
        duration_minutes: 0,
        port: 0,
        created_at: 0,
        source: "".to_string(),
        region: "".to_string(),
        status: BatchedMeasurementQuotaStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_measurement_quota("code-761", "label-504");
        assert_eq!(item.status, BatchedMeasurementQuotaStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_batched_measurement_quota("code-761", "label-504");
        if item.move_to(BatchedMeasurementQuotaStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedMeasurementQuotaStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_measurement_quota("code-761", "label-504");
        item.status = BatchedMeasurementQuotaStatus::Cancelled;
        assert!(
            item.move_to(BatchedMeasurementQuotaStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
