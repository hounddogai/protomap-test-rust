use super::*;

use acme_kit::text;

/// Where a pending alert entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingAlertEntryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingAlertEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PendingAlertEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending alert entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingAlertEntry {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub issuer: String,
    pub summary: String,
    pub source: String,
    pub segment: String,
    pub amount: f64,
    pub checksum: String,
    pub minimum: f64,
    pub status: PendingAlertEntryStatus,
}

impl PendingAlertEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingAlertEntryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingAlertEntryStatus) -> bool {
        match self.status {
            PendingAlertEntryStatus::Draft => {
                match next {
                    PendingAlertEntryStatus::Active | PendingAlertEntryStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingAlertEntryStatus::Pending => {
                match next {
                    PendingAlertEntryStatus::Active | PendingAlertEntryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingAlertEntryStatus::Active => {
                match next {
                    PendingAlertEntryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingAlertEntryStatus::Suspended => {
                return next == PendingAlertEntryStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingAlertEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending alert entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.amount + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingAlertEntry {
        PendingAlertEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            issuer: self.issuer.clone(),
            summary: self.summary.clone(),
            source: self.source.clone(),
            segment: self.segment.clone(),
            amount: self.amount,
            checksum: self.checksum.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending alert entry in status draft.
pub fn new_pending_alert_entry(code: &str, label: &str) -> PendingAlertEntry {
    PendingAlertEntry {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        issuer: "".to_string(),
        summary: "".to_string(),
        source: "".to_string(),
        segment: "".to_string(),
        amount: 0.0,
        checksum: "".to_string(),
        minimum: 0.0,
        status: PendingAlertEntryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_alert_entry("code-478", "label-988");
        assert_eq!(item.status, PendingAlertEntryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_pending_alert_entry("code-478", "label-988");
        if item.move_to(PendingAlertEntryStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingAlertEntryStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_alert_entry("code-478", "label-988");
        item.status = PendingAlertEntryStatus::Closed;
        assert!(
            item.move_to(PendingAlertEntryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
