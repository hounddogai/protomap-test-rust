use super::*;

use acme_kit::text;

/// Where a sink quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkQuotaStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SinkQuotaStatus {
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

impl std::fmt::Display for SinkQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkQuota {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub locale: String,
    pub audience: String,
    pub source: String,
    pub quantity: i64,
    pub cost: f64,
    pub created_at: i64,
    pub approved: bool,
    pub quota_units: i64,
    pub status: SinkQuotaStatus,
}

impl SinkQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkQuotaStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkQuotaStatus) -> bool {
        match self.status {
            SinkQuotaStatus::Planned => {
                match next {
                    SinkQuotaStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkQuotaStatus::Scheduled => {
                match next {
                    SinkQuotaStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkQuotaStatus::InProgress => {
                match next {
                    SinkQuotaStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkQuotaStatus::Completed => {
                match next {
                    SinkQuotaStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SinkQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink quota {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.created_at as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkQuota {
        SinkQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            locale: self.locale.clone(),
            audience: self.audience.clone(),
            source: self.source.clone(),
            quantity: self.quantity,
            cost: self.cost,
            created_at: self.created_at,
            approved: self.approved,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a sink quota in status planned.
pub fn new_sink_quota(code: &str, title: &str) -> SinkQuota {
    SinkQuota {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        locale: "".to_string(),
        audience: "".to_string(),
        source: "".to_string(),
        quantity: 0,
        cost: 0.0,
        created_at: 0,
        approved: false,
        quota_units: 0,
        status: SinkQuotaStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_sink_quota("code-842", "title-615");
        assert_eq!(item.status, SinkQuotaStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_sink_quota("code-842", "title-615");
        if item.move_to(SinkQuotaStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkQuotaStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_sink_quota("code-842", "title-615");
        item.status = SinkQuotaStatus::Cancelled;
        assert!(item.move_to(SinkQuotaStatus::Planned).is_err(), "expected move to fail");
    }
}
