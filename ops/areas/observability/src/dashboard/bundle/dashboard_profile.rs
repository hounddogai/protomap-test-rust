use super::*;

use acme_kit::text;

/// Where a dashboard profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DashboardProfileStatus {
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

impl std::fmt::Display for DashboardProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dashboard profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardProfile {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub external_ref: String,
    pub threshold: f64,
    pub contact_email: String,
    pub scope: String,
    pub cipher: String,
    pub name: String,
    pub version: i64,
    pub audience: String,
    pub status: DashboardProfileStatus,
}

impl DashboardProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardProfileStatus) -> bool {
        match self.status {
            DashboardProfileStatus::Planned => {
                return next == DashboardProfileStatus::Scheduled;
            }
            DashboardProfileStatus::Scheduled => {
                return next == DashboardProfileStatus::InProgress || next == DashboardProfileStatus::Completed;
            }
            DashboardProfileStatus::InProgress => {
                return next == DashboardProfileStatus::Completed || next == DashboardProfileStatus::Cancelled;
            }
            DashboardProfileStatus::Completed => {
                return next == DashboardProfileStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "dashboard profile", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.threshold * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardProfile {
        DashboardProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            external_ref: self.external_ref.clone(),
            threshold: self.threshold,
            contact_email: self.contact_email.clone(),
            scope: self.scope.clone(),
            cipher: self.cipher.clone(),
            name: self.name.clone(),
            version: self.version,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a dashboard profile in status planned.
pub fn new_dashboard_profile(code: &str, label: &str) -> DashboardProfile {
    DashboardProfile {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        external_ref: "".to_string(),
        threshold: 0.0,
        contact_email: "".to_string(),
        scope: "".to_string(),
        cipher: "".to_string(),
        name: "".to_string(),
        version: 0,
        audience: "".to_string(),
        status: DashboardProfileStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_dashboard_profile("code-960", "label-693");
        assert_eq!(item.status, DashboardProfileStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_dashboard_profile("code-960", "label-693");
        if item.move_to(DashboardProfileStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardProfileStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_dashboard_profile("code-960", "label-693");
        item.status = DashboardProfileStatus::Cancelled;
        assert!(
            item.move_to(DashboardProfileStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
