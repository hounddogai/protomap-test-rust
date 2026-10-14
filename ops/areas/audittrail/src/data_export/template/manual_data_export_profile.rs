use super::*;

use acme_kit::text;

/// Where a manual data export profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDataExportProfileStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualDataExportProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ManualDataExportProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual data export profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportProfile {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub retries: i64,
    pub approved: bool,
    pub country: String,
    pub description: String,
    pub timezone: String,
    pub status: ManualDataExportProfileStatus,
}

impl ManualDataExportProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDataExportProfileStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDataExportProfileStatus) -> bool {
        match self.status {
            ManualDataExportProfileStatus::Idle => {
                return next == ManualDataExportProfileStatus::Draining
                    || next == ManualDataExportProfileStatus::Stopped;
            }
            ManualDataExportProfileStatus::Warming => {
                return next == ManualDataExportProfileStatus::Stopped;
            }
            ManualDataExportProfileStatus::Ready => {
                return next == ManualDataExportProfileStatus::Stopped;
            }
            ManualDataExportProfileStatus::Draining => {
                return next == ManualDataExportProfileStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualDataExportProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual data export profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> ManualDataExportProfile {
        ManualDataExportProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            retries: self.retries,
            approved: self.approved,
            country: self.country.clone(),
            description: self.description.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a manual data export profile in status idle.
pub fn new_manual_data_export_profile(code: &str, title: &str) -> ManualDataExportProfile {
    ManualDataExportProfile {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        retries: 0,
        approved: false,
        country: "".to_string(),
        description: "".to_string(),
        timezone: "".to_string(),
        status: ManualDataExportProfileStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_data_export_profile("code-676", "title-841");
        assert_eq!(item.status, ManualDataExportProfileStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_manual_data_export_profile("code-676", "title-841");
        if item.move_to(ManualDataExportProfileStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualDataExportProfileStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_data_export_profile("code-676", "title-841");
        item.status = ManualDataExportProfileStatus::Stopped;
        assert!(
            item.move_to(ManualDataExportProfileStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
