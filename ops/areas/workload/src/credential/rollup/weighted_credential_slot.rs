use super::*;

use acme_kit::text;

/// Where a weighted credential slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedCredentialSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedCredentialSlotStatus {
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

impl std::fmt::Display for WeightedCredentialSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted credential slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialSlot {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub summary: String,
    pub ratio: f64,
    pub quantity: i64,
    pub expires_at: i64,
    pub limit: i64,
    pub contact_email: String,
    pub status: WeightedCredentialSlotStatus,
}

impl WeightedCredentialSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedCredentialSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedCredentialSlotStatus) -> bool {
        match self.status {
            WeightedCredentialSlotStatus::Open => {
                return next == WeightedCredentialSlotStatus::Reopened;
            }
            WeightedCredentialSlotStatus::InReview => {
                match next {
                    WeightedCredentialSlotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedCredentialSlotStatus::Resolved => {
                return next == WeightedCredentialSlotStatus::Reopened
                    || next == WeightedCredentialSlotStatus::Archived;
            }
            WeightedCredentialSlotStatus::Reopened => {
                return next == WeightedCredentialSlotStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedCredentialSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted credential slot {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.limit as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedCredentialSlot {
        WeightedCredentialSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            summary: self.summary.clone(),
            ratio: self.ratio,
            quantity: self.quantity,
            expires_at: self.expires_at,
            limit: self.limit,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted credential slot in status open.
pub fn new_weighted_credential_slot(code: &str, label: &str) -> WeightedCredentialSlot {
    WeightedCredentialSlot {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        summary: "".to_string(),
        ratio: 0.0,
        quantity: 0,
        expires_at: 0,
        limit: 0,
        contact_email: "".to_string(),
        status: WeightedCredentialSlotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_weighted_credential_slot("code-961", "label-685");
        assert_eq!(item.status, WeightedCredentialSlotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_weighted_credential_slot("code-961", "label-685");
        if item.move_to(WeightedCredentialSlotStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedCredentialSlotStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_weighted_credential_slot("code-961", "label-685");
        item.status = WeightedCredentialSlotStatus::Archived;
        assert!(
            item.move_to(WeightedCredentialSlotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
