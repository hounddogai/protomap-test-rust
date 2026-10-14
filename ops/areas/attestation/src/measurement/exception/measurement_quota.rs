use super::*;

use acme_kit::text;

/// Where a measurement quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementQuotaStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl MeasurementQuotaStatus {
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

impl std::fmt::Display for MeasurementQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a measurement quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementQuota {
    pub code: String,
    pub title: String,
    pub name: String,
    pub description: String,
    pub attempts: i64,
    pub minimum: f64,
    pub created_at: i64,
    pub external_ref: String,
    pub audience: String,
    pub tenant_id: String,
    pub status: MeasurementQuotaStatus,
}

impl MeasurementQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementQuotaStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementQuotaStatus) -> bool {
        match self.status {
            MeasurementQuotaStatus::Planned => {
                return next == MeasurementQuotaStatus::Completed;
            }
            MeasurementQuotaStatus::Scheduled => {
                return next == MeasurementQuotaStatus::Cancelled;
            }
            MeasurementQuotaStatus::InProgress => {
                return next == MeasurementQuotaStatus::Completed || next == MeasurementQuotaStatus::Cancelled;
            }
            MeasurementQuotaStatus::Completed => {
                match next {
                    MeasurementQuotaStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: MeasurementQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement quota {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.created_at as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementQuota {
        MeasurementQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            attempts: self.attempts,
            minimum: self.minimum,
            created_at: self.created_at,
            external_ref: self.external_ref.clone(),
            audience: self.audience.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a measurement quota in status planned.
pub fn new_measurement_quota(code: &str, title: &str) -> MeasurementQuota {
    MeasurementQuota {
        code: code.to_string(),
        title: title.to_string(),
        name: "".to_string(),
        description: "".to_string(),
        attempts: 0,
        minimum: 0.0,
        created_at: 0,
        external_ref: "".to_string(),
        audience: "".to_string(),
        tenant_id: "".to_string(),
        status: MeasurementQuotaStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_measurement_quota("code-17", "title-360");
        assert_eq!(item.status, MeasurementQuotaStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_measurement_quota("code-17", "title-360");
        if item.move_to(MeasurementQuotaStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementQuotaStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_measurement_quota("code-17", "title-360");
        item.status = MeasurementQuotaStatus::Cancelled;
        assert!(
            item.move_to(MeasurementQuotaStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
