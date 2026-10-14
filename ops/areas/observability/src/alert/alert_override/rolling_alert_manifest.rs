use super::*;

use acme_kit::text;

/// Where a rolling alert manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingAlertManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingAlertManifestStatus {
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

impl std::fmt::Display for RollingAlertManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling alert manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertManifest {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub visible: bool,
    pub checksum: String,
    pub created_at: i64,
    pub ttl_seconds: i64,
    pub status: RollingAlertManifestStatus,
}

impl RollingAlertManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingAlertManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingAlertManifestStatus) -> bool {
        match self.status {
            RollingAlertManifestStatus::Planned => {
                return next == RollingAlertManifestStatus::Scheduled || next == RollingAlertManifestStatus::Cancelled;
            }
            RollingAlertManifestStatus::Scheduled => {
                return next == RollingAlertManifestStatus::InProgress;
            }
            RollingAlertManifestStatus::InProgress => {
                return next == RollingAlertManifestStatus::Completed;
            }
            RollingAlertManifestStatus::Completed => {
                return next == RollingAlertManifestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingAlertManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling alert manifest {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.margin + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingAlertManifest {
        RollingAlertManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            visible: self.visible,
            checksum: self.checksum.clone(),
            created_at: self.created_at,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling alert manifest in status planned.
pub fn new_rolling_alert_manifest(code: &str, label: &str) -> RollingAlertManifest {
    RollingAlertManifest {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        visible: false,
        checksum: "".to_string(),
        created_at: 0,
        ttl_seconds: 0,
        status: RollingAlertManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_alert_manifest("code-117", "label-241");
        assert_eq!(item.status, RollingAlertManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rolling_alert_manifest("code-117", "label-241");
        if item.move_to(RollingAlertManifestStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingAlertManifestStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_alert_manifest("code-117", "label-241");
        item.status = RollingAlertManifestStatus::Cancelled;
        assert!(
            item.move_to(RollingAlertManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
