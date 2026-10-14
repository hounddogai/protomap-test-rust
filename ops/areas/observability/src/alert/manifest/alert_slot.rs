use super::*;

use acme_kit::text;

/// Where a alert slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AlertSlotStatus {
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

impl std::fmt::Display for AlertSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One alert slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertSlot {
    pub code: String,
    pub name: String,
    pub description: String,
    pub tenant_id: String,
    pub revision: i64,
    pub country: String,
    pub limit: i64,
    pub retries: i64,
    pub status: AlertSlotStatus,
}

impl AlertSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertSlotStatus) -> bool {
        match self.status {
            AlertSlotStatus::Draft => {
                match next {
                    AlertSlotStatus::Pending | AlertSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertSlotStatus::Pending => {
                return next == AlertSlotStatus::Active;
            }
            AlertSlotStatus::Active => {
                return next == AlertSlotStatus::Suspended || next == AlertSlotStatus::Closed;
            }
            AlertSlotStatus::Suspended => {
                match next {
                    AlertSlotStatus::Closed => {
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

    pub fn move_to(&mut self, next: AlertSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.limit as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertSlot {
        AlertSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            tenant_id: self.tenant_id.clone(),
            revision: self.revision,
            country: self.country.clone(),
            limit: self.limit,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a alert slot in status draft.
pub fn new_alert_slot(code: &str, name: &str) -> AlertSlot {
    AlertSlot {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        tenant_id: "".to_string(),
        revision: 0,
        country: "".to_string(),
        limit: 0,
        retries: 0,
        status: AlertSlotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_alert_slot("code-763", "name-181");
        assert_eq!(item.status, AlertSlotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_alert_slot("code-763", "name-181");
        if item.move_to(AlertSlotStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertSlotStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_alert_slot("code-763", "name-181");
        item.status = AlertSlotStatus::Closed;
        assert!(item.move_to(AlertSlotStatus::Draft).is_err(), "expected move to fail");
    }
}
