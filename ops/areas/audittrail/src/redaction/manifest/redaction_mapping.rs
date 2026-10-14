use super::*;

use acme_kit::text;

/// Where a redaction mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RedactionMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RedactionMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One redaction mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMapping {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub quantity: i64,
    pub threshold: f64,
    pub status: RedactionMappingStatus,
}

impl RedactionMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionMappingStatus) -> bool {
        match self.status {
            RedactionMappingStatus::Proposed => {
                match next {
                    RedactionMappingStatus::Approved | RedactionMappingStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionMappingStatus::Approved => {
                return next == RedactionMappingStatus::Rejected || next == RedactionMappingStatus::Withdrawn;
            }
            RedactionMappingStatus::Rejected => {
                return next == RedactionMappingStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "redaction mapping", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.threshold - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionMapping {
        RedactionMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            quantity: self.quantity,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a redaction mapping in status proposed.
pub fn new_redaction_mapping(code: &str, title: &str) -> RedactionMapping {
    RedactionMapping {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        quantity: 0,
        threshold: 0.0,
        status: RedactionMappingStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_redaction_mapping("code-384", "title-449");
        assert_eq!(item.status, RedactionMappingStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_redaction_mapping("code-384", "title-449");
        if item.move_to(RedactionMappingStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionMappingStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_redaction_mapping("code-384", "title-449");
        item.status = RedactionMappingStatus::Withdrawn;
        assert!(
            item.move_to(RedactionMappingStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
