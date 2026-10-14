use super::*;

use acme_kit::text;

/// Where a primary sink slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySinkSlotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimarySinkSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimarySinkSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary sink slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkSlot {
    pub code: String,
    pub name: String,
    pub category: String,
    pub approved: bool,
    pub principal: String,
    pub contact_email: String,
    pub cipher: String,
    pub score: f64,
    pub status: PrimarySinkSlotStatus,
}

impl PrimarySinkSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySinkSlotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySinkSlotStatus) -> bool {
        match self.status {
            PrimarySinkSlotStatus::Created => {
                return next == PrimarySinkSlotStatus::Retired;
            }
            PrimarySinkSlotStatus::Validated => {
                return next == PrimarySinkSlotStatus::Published || next == PrimarySinkSlotStatus::Retired;
            }
            PrimarySinkSlotStatus::Published => {
                match next {
                    PrimarySinkSlotStatus::Retired => {
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

    pub fn move_to(&mut self, next: PrimarySinkSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary sink slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> PrimarySinkSlot {
        PrimarySinkSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            approved: self.approved,
            principal: self.principal.clone(),
            contact_email: self.contact_email.clone(),
            cipher: self.cipher.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a primary sink slot in status created.
pub fn new_primary_sink_slot(code: &str, name: &str) -> PrimarySinkSlot {
    PrimarySinkSlot {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        approved: false,
        principal: "".to_string(),
        contact_email: "".to_string(),
        cipher: "".to_string(),
        score: 0.0,
        status: PrimarySinkSlotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_primary_sink_slot("code-711", "name-497");
        assert_eq!(item.status, PrimarySinkSlotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_primary_sink_slot("code-711", "name-497");
        if item.move_to(PrimarySinkSlotStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySinkSlotStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_primary_sink_slot("code-711", "name-497");
        item.status = PrimarySinkSlotStatus::Retired;
        assert!(
            item.move_to(PrimarySinkSlotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
