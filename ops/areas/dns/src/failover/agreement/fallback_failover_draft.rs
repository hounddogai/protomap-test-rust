use super::*;

use acme_kit::text;

/// Where a fallback failover draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackFailoverDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackFailoverDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackFailoverDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback failover draft as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverDraft {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub visible: bool,
    pub country: String,
    pub key_id: String,
    pub active: bool,
    pub phone: String,
    pub score: f64,
    pub status: FallbackFailoverDraftStatus,
}

impl FallbackFailoverDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackFailoverDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackFailoverDraftStatus) -> bool {
        match self.status {
            FallbackFailoverDraftStatus::Created => {
                match next {
                    FallbackFailoverDraftStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFailoverDraftStatus::Validated => {
                match next {
                    FallbackFailoverDraftStatus::Published | FallbackFailoverDraftStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFailoverDraftStatus::Published => {
                match next {
                    FallbackFailoverDraftStatus::Retired => {
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

    pub fn move_to(&mut self, next: FallbackFailoverDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback failover draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback failover draft", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> FallbackFailoverDraft {
        FallbackFailoverDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            visible: self.visible,
            country: self.country.clone(),
            key_id: self.key_id.clone(),
            active: self.active,
            phone: self.phone.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a fallback failover draft in status created.
pub fn new_fallback_failover_draft(code: &str, label: &str) -> FallbackFailoverDraft {
    FallbackFailoverDraft {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        visible: false,
        country: "".to_string(),
        key_id: "".to_string(),
        active: false,
        phone: "".to_string(),
        score: 0.0,
        status: FallbackFailoverDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_failover_draft("code-750", "label-895");
        assert_eq!(item.status, FallbackFailoverDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_fallback_failover_draft("code-750", "label-895");
        if item.move_to(FallbackFailoverDraftStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackFailoverDraftStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_failover_draft("code-750", "label-895");
        item.status = FallbackFailoverDraftStatus::Retired;
        assert!(
            item.move_to(FallbackFailoverDraftStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
