use super::*;

use acme_kit::text;

/// Where a primary overage entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryOverageEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryOverageEntryStatus {
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

impl std::fmt::Display for PrimaryOverageEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary overage entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageEntry {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub limit: i64,
    pub issuer: String,
    pub external_ref: String,
    pub quota_units: i64,
    pub unit: String,
    pub host: String,
    pub archived: bool,
    pub status: PrimaryOverageEntryStatus,
}

impl PrimaryOverageEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryOverageEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryOverageEntryStatus) -> bool {
        match self.status {
            PrimaryOverageEntryStatus::Planned => {
                return next == PrimaryOverageEntryStatus::Cancelled;
            }
            PrimaryOverageEntryStatus::Scheduled => {
                return next == PrimaryOverageEntryStatus::InProgress;
            }
            PrimaryOverageEntryStatus::InProgress => {
                return next == PrimaryOverageEntryStatus::Completed;
            }
            PrimaryOverageEntryStatus::Completed => {
                match next {
                    PrimaryOverageEntryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PrimaryOverageEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary overage entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.quota_units as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryOverageEntry {
        PrimaryOverageEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            limit: self.limit,
            issuer: self.issuer.clone(),
            external_ref: self.external_ref.clone(),
            quota_units: self.quota_units,
            unit: self.unit.clone(),
            host: self.host.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary overage entry in status planned.
pub fn new_primary_overage_entry(code: &str, label: &str) -> PrimaryOverageEntry {
    PrimaryOverageEntry {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        limit: 0,
        issuer: "".to_string(),
        external_ref: "".to_string(),
        quota_units: 0,
        unit: "".to_string(),
        host: "".to_string(),
        archived: false,
        status: PrimaryOverageEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_overage_entry("code-186", "label-953");
        assert_eq!(item.status, PrimaryOverageEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_primary_overage_entry("code-186", "label-953");
        if item.move_to(PrimaryOverageEntryStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryOverageEntryStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_overage_entry("code-186", "label-953");
        item.status = PrimaryOverageEntryStatus::Cancelled;
        assert!(
            item.move_to(PrimaryOverageEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
