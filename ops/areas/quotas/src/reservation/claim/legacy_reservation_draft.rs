use super::*;

use acme_kit::text;

/// Where a legacy reservation draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyReservationDraftStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyReservationDraftStatus {
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

impl std::fmt::Display for LegacyReservationDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy reservation draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationDraft {
    pub code: String,
    pub label: String,
    pub title: String,
    pub tenant_id: String,
    pub visible: bool,
    pub timezone: String,
    pub quantity: i64,
    pub status: LegacyReservationDraftStatus,
}

impl LegacyReservationDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyReservationDraftStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyReservationDraftStatus) -> bool {
        match self.status {
            LegacyReservationDraftStatus::Draft => {
                return next == LegacyReservationDraftStatus::Suspended;
            }
            LegacyReservationDraftStatus::Pending => {
                return next == LegacyReservationDraftStatus::Active || next == LegacyReservationDraftStatus::Closed;
            }
            LegacyReservationDraftStatus::Active => {
                match next {
                    LegacyReservationDraftStatus::Suspended | LegacyReservationDraftStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyReservationDraftStatus::Suspended => {
                match next {
                    LegacyReservationDraftStatus::Closed => {
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

    pub fn move_to(&mut self, next: LegacyReservationDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy reservation draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> LegacyReservationDraft {
        LegacyReservationDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            visible: self.visible,
            timezone: self.timezone.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy reservation draft in status draft.
pub fn new_legacy_reservation_draft(code: &str, label: &str) -> LegacyReservationDraft {
    LegacyReservationDraft {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        tenant_id: "".to_string(),
        visible: false,
        timezone: "".to_string(),
        quantity: 0,
        status: LegacyReservationDraftStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_reservation_draft("code-838", "label-200");
        assert_eq!(item.status, LegacyReservationDraftStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_legacy_reservation_draft("code-838", "label-200");
        if item.move_to(LegacyReservationDraftStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyReservationDraftStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_reservation_draft("code-838", "label-200");
        item.status = LegacyReservationDraftStatus::Closed;
        assert!(
            item.move_to(LegacyReservationDraftStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
