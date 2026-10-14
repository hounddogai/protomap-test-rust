use super::*;

use acme_kit::text;

/// Where a allowlist reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AllowlistReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AllowlistReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistReservation {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub cluster: String,
    pub visible: bool,
    pub key_id: String,
    pub description: String,
    pub owner_id: String,
    pub amount: f64,
    pub title: String,
    pub tenant_id: String,
    pub status: AllowlistReservationStatus,
}

impl AllowlistReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistReservationStatus) -> bool {
        match self.status {
            AllowlistReservationStatus::Created => {
                return next == AllowlistReservationStatus::Published;
            }
            AllowlistReservationStatus::Validated => {
                return next == AllowlistReservationStatus::Published || next == AllowlistReservationStatus::Retired;
            }
            AllowlistReservationStatus::Published => {
                return next == AllowlistReservationStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "allowlist reservation", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.amount + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistReservation {
        AllowlistReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            cluster: self.cluster.clone(),
            visible: self.visible,
            key_id: self.key_id.clone(),
            description: self.description.clone(),
            owner_id: self.owner_id.clone(),
            amount: self.amount,
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a allowlist reservation in status created.
pub fn new_allowlist_reservation(code: &str, label: &str) -> AllowlistReservation {
    AllowlistReservation {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        cluster: "".to_string(),
        visible: false,
        key_id: "".to_string(),
        description: "".to_string(),
        owner_id: "".to_string(),
        amount: 0.0,
        title: "".to_string(),
        tenant_id: "".to_string(),
        status: AllowlistReservationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_allowlist_reservation("code-641", "label-838");
        assert_eq!(item.status, AllowlistReservationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_allowlist_reservation("code-641", "label-838");
        if item.move_to(AllowlistReservationStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistReservationStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_allowlist_reservation("code-641", "label-838");
        item.status = AllowlistReservationStatus::Retired;
        assert!(
            item.move_to(AllowlistReservationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
