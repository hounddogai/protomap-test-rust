use super::*;

use acme_kit::text;

/// Where a identity slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentitySlotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl IdentitySlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for IdentitySlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a identity slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySlot {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub locale: String,
    pub issuer: String,
    pub scope: String,
    pub channel: String,
    pub updated_at: i64,
    pub status: IdentitySlotStatus,
}

impl IdentitySlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentitySlotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentitySlotStatus) -> bool {
        match self.status {
            IdentitySlotStatus::Created => {
                match next {
                    IdentitySlotStatus::Published | IdentitySlotStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentitySlotStatus::Validated => {
                return next == IdentitySlotStatus::Published || next == IdentitySlotStatus::Retired;
            }
            IdentitySlotStatus::Published => {
                return next == IdentitySlotStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentitySlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity slot {} cannot move from {} to {}",
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
        format!("{} {}", "identity slot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> IdentitySlot {
        IdentitySlot {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            locale: self.locale.clone(),
            issuer: self.issuer.clone(),
            scope: self.scope.clone(),
            channel: self.channel.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a identity slot in status created.
pub fn new_identity_slot(code: &str, label: &str) -> IdentitySlot {
    IdentitySlot {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        locale: "".to_string(),
        issuer: "".to_string(),
        scope: "".to_string(),
        channel: "".to_string(),
        updated_at: 0,
        status: IdentitySlotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_identity_slot("code-750", "label-632");
        assert_eq!(item.status, IdentitySlotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_identity_slot("code-750", "label-632");
        if item.move_to(IdentitySlotStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentitySlotStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_identity_slot("code-750", "label-632");
        item.status = IdentitySlotStatus::Retired;
        assert!(
            item.move_to(IdentitySlotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
