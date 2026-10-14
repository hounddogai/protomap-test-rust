use super::*;

use acme_kit::text;

/// Where a fallback action item override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackActionItemOverrideStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FallbackActionItemOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FallbackActionItemOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback action item override as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemOverride {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub namespace: String,
    pub sequence: i64,
    pub ends_at: i64,
    pub locale: String,
    pub maximum: f64,
    pub status: FallbackActionItemOverrideStatus,
}

impl FallbackActionItemOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackActionItemOverrideStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackActionItemOverrideStatus) -> bool {
        match self.status {
            FallbackActionItemOverrideStatus::Proposed => {
                match next {
                    FallbackActionItemOverrideStatus::Approved | FallbackActionItemOverrideStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackActionItemOverrideStatus::Approved => {
                match next {
                    FallbackActionItemOverrideStatus::Rejected | FallbackActionItemOverrideStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackActionItemOverrideStatus::Rejected => {
                match next {
                    FallbackActionItemOverrideStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FallbackActionItemOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback action item override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        (self.maximum * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackActionItemOverride {
        FallbackActionItemOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            namespace: self.namespace.clone(),
            sequence: self.sequence,
            ends_at: self.ends_at,
            locale: self.locale.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback action item override in status proposed.
pub fn new_fallback_action_item_override(code: &str, title: &str) -> FallbackActionItemOverride {
    FallbackActionItemOverride {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        namespace: "".to_string(),
        sequence: 0,
        ends_at: 0,
        locale: "".to_string(),
        maximum: 0.0,
        status: FallbackActionItemOverrideStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_fallback_action_item_override("code-696", "title-316");
        assert_eq!(item.status, FallbackActionItemOverrideStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_fallback_action_item_override("code-696", "title-316");
        if item.move_to(FallbackActionItemOverrideStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackActionItemOverrideStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_fallback_action_item_override("code-696", "title-316");
        item.status = FallbackActionItemOverrideStatus::Withdrawn;
        assert!(
            item.move_to(FallbackActionItemOverrideStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
