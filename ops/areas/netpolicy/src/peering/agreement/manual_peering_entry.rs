use super::*;

use acme_kit::text;

/// Where a manual peering entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualPeeringEntryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualPeeringEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ManualPeeringEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual peering entry as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringEntry {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub fingerprint: String,
    pub visible: bool,
    pub created_at: i64,
    pub window_days: i64,
    pub status: ManualPeeringEntryStatus,
}

impl ManualPeeringEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualPeeringEntryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualPeeringEntryStatus) -> bool {
        match self.status {
            ManualPeeringEntryStatus::Open => {
                return next == ManualPeeringEntryStatus::InReview || next == ManualPeeringEntryStatus::Resolved;
            }
            ManualPeeringEntryStatus::InReview => {
                match next {
                    ManualPeeringEntryStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualPeeringEntryStatus::Resolved => {
                match next {
                    ManualPeeringEntryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualPeeringEntryStatus::Reopened => {
                return next == ManualPeeringEntryStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualPeeringEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual peering entry {} cannot move from {} to {}",
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
        format!("{} {}", "manual peering entry", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.window_days as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualPeeringEntry {
        ManualPeeringEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            fingerprint: self.fingerprint.clone(),
            visible: self.visible,
            created_at: self.created_at,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a manual peering entry in status open.
pub fn new_manual_peering_entry(code: &str, label: &str) -> ManualPeeringEntry {
    ManualPeeringEntry {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        fingerprint: "".to_string(),
        visible: false,
        created_at: 0,
        window_days: 0,
        status: ManualPeeringEntryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_peering_entry("code-192", "label-805");
        assert_eq!(item.status, ManualPeeringEntryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_manual_peering_entry("code-192", "label-805");
        if item.move_to(ManualPeeringEntryStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualPeeringEntryStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_peering_entry("code-192", "label-805");
        item.status = ManualPeeringEntryStatus::Archived;
        assert!(
            item.move_to(ManualPeeringEntryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
