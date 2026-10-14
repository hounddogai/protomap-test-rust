use super::*;

use acme_kit::text;

/// Where a legacy binding slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyBindingSlotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyBindingSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyBindingSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy binding slot as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingSlot {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub rate: f64,
    pub cluster: String,
    pub status: LegacyBindingSlotStatus,
}

impl LegacyBindingSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyBindingSlotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyBindingSlotStatus) -> bool {
        match self.status {
            LegacyBindingSlotStatus::Created => {
                return next == LegacyBindingSlotStatus::Validated || next == LegacyBindingSlotStatus::Retired;
            }
            LegacyBindingSlotStatus::Validated => {
                match next {
                    LegacyBindingSlotStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyBindingSlotStatus::Published => {
                return next == LegacyBindingSlotStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyBindingSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy binding slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.rate
    }

    pub fn copy(&self) -> LegacyBindingSlot {
        LegacyBindingSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            rate: self.rate,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a legacy binding slot in status created.
pub fn new_legacy_binding_slot(code: &str, name: &str) -> LegacyBindingSlot {
    LegacyBindingSlot {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        rate: 0.0,
        cluster: "".to_string(),
        status: LegacyBindingSlotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_binding_slot("code-309", "name-1");
        assert_eq!(item.status, LegacyBindingSlotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_legacy_binding_slot("code-309", "name-1");
        if item.move_to(LegacyBindingSlotStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyBindingSlotStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_binding_slot("code-309", "name-1");
        item.status = LegacyBindingSlotStatus::Retired;
        assert!(
            item.move_to(LegacyBindingSlotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
