use super::*;

use acme_kit::text;

/// Where a legacy alert slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyAlertSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyAlertSlotStatus {
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

impl std::fmt::Display for LegacyAlertSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy alert slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAlertSlot {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub external_ref: String,
    pub version: i64,
    pub expires_at: i64,
    pub country: String,
    pub status: LegacyAlertSlotStatus,
}

impl LegacyAlertSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyAlertSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyAlertSlotStatus) -> bool {
        match self.status {
            LegacyAlertSlotStatus::Draft => {
                match next {
                    LegacyAlertSlotStatus::Active | LegacyAlertSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAlertSlotStatus::Pending => {
                match next {
                    LegacyAlertSlotStatus::Active | LegacyAlertSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAlertSlotStatus::Active => {
                match next {
                    LegacyAlertSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAlertSlotStatus::Suspended => {
                return next == LegacyAlertSlotStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyAlertSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy alert slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy alert slot", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.expires_at as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyAlertSlot {
        LegacyAlertSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            external_ref: self.external_ref.clone(),
            version: self.version,
            expires_at: self.expires_at,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a legacy alert slot in status draft.
pub fn new_legacy_alert_slot(code: &str, name: &str) -> LegacyAlertSlot {
    LegacyAlertSlot {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        external_ref: "".to_string(),
        version: 0,
        expires_at: 0,
        country: "".to_string(),
        status: LegacyAlertSlotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_alert_slot("code-787", "name-595");
        assert_eq!(item.status, LegacyAlertSlotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_legacy_alert_slot("code-787", "name-595");
        if item.move_to(LegacyAlertSlotStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyAlertSlotStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_alert_slot("code-787", "name-595");
        item.status = LegacyAlertSlotStatus::Closed;
        assert!(
            item.move_to(LegacyAlertSlotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
