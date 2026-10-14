use super::*;

use acme_kit::text;

/// Where a adaptive page snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptivePageSnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptivePageSnapshotStatus {
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

impl std::fmt::Display for AdaptivePageSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive page snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePageSnapshot {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub archived: bool,
    pub currency: String,
    pub key_id: String,
    pub burst: i64,
    pub ratio: f64,
    pub external_ref: String,
    pub sequence: i64,
    pub discount: f64,
    pub status: AdaptivePageSnapshotStatus,
}

impl AdaptivePageSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptivePageSnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptivePageSnapshotStatus) -> bool {
        match self.status {
            AdaptivePageSnapshotStatus::Planned => {
                match next {
                    AdaptivePageSnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptivePageSnapshotStatus::Scheduled => {
                match next {
                    AdaptivePageSnapshotStatus::Completed | AdaptivePageSnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptivePageSnapshotStatus::InProgress => {
                return next == AdaptivePageSnapshotStatus::Completed;
            }
            AdaptivePageSnapshotStatus::Completed => {
                return next == AdaptivePageSnapshotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptivePageSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive page snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive page snapshot", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.discount - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptivePageSnapshot {
        AdaptivePageSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            archived: self.archived,
            currency: self.currency.clone(),
            key_id: self.key_id.clone(),
            burst: self.burst,
            ratio: self.ratio,
            external_ref: self.external_ref.clone(),
            sequence: self.sequence,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive page snapshot in status planned.
pub fn new_adaptive_page_snapshot(code: &str, label: &str) -> AdaptivePageSnapshot {
    AdaptivePageSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        archived: false,
        currency: "".to_string(),
        key_id: "".to_string(),
        burst: 0,
        ratio: 0.0,
        external_ref: "".to_string(),
        sequence: 0,
        discount: 0.0,
        status: AdaptivePageSnapshotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_adaptive_page_snapshot("code-994", "label-732");
        assert_eq!(item.status, AdaptivePageSnapshotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_adaptive_page_snapshot("code-994", "label-732");
        if item.move_to(AdaptivePageSnapshotStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptivePageSnapshotStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_adaptive_page_snapshot("code-994", "label-732");
        item.status = AdaptivePageSnapshotStatus::Cancelled;
        assert!(
            item.move_to(AdaptivePageSnapshotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
