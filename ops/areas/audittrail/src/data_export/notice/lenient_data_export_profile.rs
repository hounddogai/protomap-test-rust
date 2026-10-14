use super::*;

use acme_kit::text;

/// Where a lenient data export profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientDataExportProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientDataExportProfileStatus {
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

impl std::fmt::Display for LenientDataExportProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient data export profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDataExportProfile {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub burst: i64,
    pub namespace: String,
    pub segment: String,
    pub margin: f64,
    pub pinned: bool,
    pub updated_at: i64,
    pub category: String,
    pub minimum: f64,
    pub status: LenientDataExportProfileStatus,
}

impl LenientDataExportProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientDataExportProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientDataExportProfileStatus) -> bool {
        match self.status {
            LenientDataExportProfileStatus::Planned => {
                return next == LenientDataExportProfileStatus::Scheduled
                    || next == LenientDataExportProfileStatus::InProgress;
            }
            LenientDataExportProfileStatus::Scheduled => {
                match next {
                    LenientDataExportProfileStatus::Completed | LenientDataExportProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientDataExportProfileStatus::InProgress => {
                match next {
                    LenientDataExportProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientDataExportProfileStatus::Completed => {
                return next == LenientDataExportProfileStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientDataExportProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient data export profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.margin + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientDataExportProfile {
        LenientDataExportProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            burst: self.burst,
            namespace: self.namespace.clone(),
            segment: self.segment.clone(),
            margin: self.margin,
            pinned: self.pinned,
            updated_at: self.updated_at,
            category: self.category.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a lenient data export profile in status planned.
pub fn new_lenient_data_export_profile(code: &str, label: &str) -> LenientDataExportProfile {
    LenientDataExportProfile {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        burst: 0,
        namespace: "".to_string(),
        segment: "".to_string(),
        margin: 0.0,
        pinned: false,
        updated_at: 0,
        category: "".to_string(),
        minimum: 0.0,
        status: LenientDataExportProfileStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_data_export_profile("code-532", "label-948");
        assert_eq!(item.status, LenientDataExportProfileStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_lenient_data_export_profile("code-532", "label-948");
        if item.move_to(LenientDataExportProfileStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientDataExportProfileStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_data_export_profile("code-532", "label-948");
        item.status = LenientDataExportProfileStatus::Cancelled;
        assert!(
            item.move_to(LenientDataExportProfileStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
