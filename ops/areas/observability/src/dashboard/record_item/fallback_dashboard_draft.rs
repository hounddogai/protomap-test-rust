use super::*;

use acme_kit::text;

/// Where a fallback dashboard draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackDashboardDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackDashboardDraftStatus {
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

impl std::fmt::Display for FallbackDashboardDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback dashboard draft as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardDraft {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub reviewer: String,
    pub owner_id: String,
    pub batch_size: i64,
    pub weight: f64,
    pub ttl_seconds: i64,
    pub margin: f64,
    pub host: String,
    pub status: FallbackDashboardDraftStatus,
}

impl FallbackDashboardDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackDashboardDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackDashboardDraftStatus) -> bool {
        match self.status {
            FallbackDashboardDraftStatus::Planned => {
                match next {
                    FallbackDashboardDraftStatus::Completed | FallbackDashboardDraftStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDashboardDraftStatus::Scheduled => {
                match next {
                    FallbackDashboardDraftStatus::InProgress | FallbackDashboardDraftStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDashboardDraftStatus::InProgress => {
                return next == FallbackDashboardDraftStatus::Completed
                    || next == FallbackDashboardDraftStatus::Cancelled;
            }
            FallbackDashboardDraftStatus::Completed => {
                return next == FallbackDashboardDraftStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackDashboardDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback dashboard draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackDashboardDraft {
        FallbackDashboardDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            reviewer: self.reviewer.clone(),
            owner_id: self.owner_id.clone(),
            batch_size: self.batch_size,
            weight: self.weight,
            ttl_seconds: self.ttl_seconds,
            margin: self.margin,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback dashboard draft in status planned.
pub fn new_fallback_dashboard_draft(code: &str, label: &str) -> FallbackDashboardDraft {
    FallbackDashboardDraft {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        reviewer: "".to_string(),
        owner_id: "".to_string(),
        batch_size: 0,
        weight: 0.0,
        ttl_seconds: 0,
        margin: 0.0,
        host: "".to_string(),
        status: FallbackDashboardDraftStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_dashboard_draft("code-904", "label-707");
        assert_eq!(item.status, FallbackDashboardDraftStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_fallback_dashboard_draft("code-904", "label-707");
        if item.move_to(FallbackDashboardDraftStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackDashboardDraftStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_dashboard_draft("code-904", "label-707");
        item.status = FallbackDashboardDraftStatus::Cancelled;
        assert!(
            item.move_to(FallbackDashboardDraftStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
