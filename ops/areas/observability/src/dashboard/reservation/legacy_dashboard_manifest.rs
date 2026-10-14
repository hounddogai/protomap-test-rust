use super::*;

use acme_kit::text;

/// Where a legacy dashboard manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyDashboardManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyDashboardManifestStatus {
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

impl std::fmt::Display for LegacyDashboardManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy dashboard manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDashboardManifest {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub discount: f64,
    pub active: bool,
    pub ratio: f64,
    pub country: String,
    pub principal: String,
    pub source: String,
    pub phone: String,
    pub status: LegacyDashboardManifestStatus,
}

impl LegacyDashboardManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyDashboardManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyDashboardManifestStatus) -> bool {
        match self.status {
            LegacyDashboardManifestStatus::Planned => {
                match next {
                    LegacyDashboardManifestStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyDashboardManifestStatus::Scheduled => {
                return next == LegacyDashboardManifestStatus::Cancelled;
            }
            LegacyDashboardManifestStatus::InProgress => {
                match next {
                    LegacyDashboardManifestStatus::Completed | LegacyDashboardManifestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyDashboardManifestStatus::Completed => {
                return next == LegacyDashboardManifestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyDashboardManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy dashboard manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy dashboard manifest", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ratio - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyDashboardManifest {
        LegacyDashboardManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            discount: self.discount,
            active: self.active,
            ratio: self.ratio,
            country: self.country.clone(),
            principal: self.principal.clone(),
            source: self.source.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a legacy dashboard manifest in status planned.
pub fn new_legacy_dashboard_manifest(code: &str, title: &str) -> LegacyDashboardManifest {
    LegacyDashboardManifest {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        discount: 0.0,
        active: false,
        ratio: 0.0,
        country: "".to_string(),
        principal: "".to_string(),
        source: "".to_string(),
        phone: "".to_string(),
        status: LegacyDashboardManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_legacy_dashboard_manifest("code-644", "title-3");
        assert_eq!(item.status, LegacyDashboardManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_legacy_dashboard_manifest("code-644", "title-3");
        if item.move_to(LegacyDashboardManifestStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyDashboardManifestStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_legacy_dashboard_manifest("code-644", "title-3");
        item.status = LegacyDashboardManifestStatus::Cancelled;
        assert!(
            item.move_to(LegacyDashboardManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
