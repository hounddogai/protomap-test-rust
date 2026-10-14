use super::*;

use acme_kit::text;

/// Where a fallback action item digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackActionItemDigestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackActionItemDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackActionItemDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback action item digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemDigest {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub rotation_days: i64,
    pub notes: String,
    pub grace_days: i64,
    pub status: FallbackActionItemDigestStatus,
}

impl FallbackActionItemDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackActionItemDigestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackActionItemDigestStatus) -> bool {
        match self.status {
            FallbackActionItemDigestStatus::Created => {
                return next == FallbackActionItemDigestStatus::Retired;
            }
            FallbackActionItemDigestStatus::Validated => {
                return next == FallbackActionItemDigestStatus::Published
                    || next == FallbackActionItemDigestStatus::Retired;
            }
            FallbackActionItemDigestStatus::Published => {
                match next {
                    FallbackActionItemDigestStatus::Retired => {
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

    pub fn move_to(&mut self, next: FallbackActionItemDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback action item digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.rotation_days as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackActionItemDigest {
        FallbackActionItemDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            rotation_days: self.rotation_days,
            notes: self.notes.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback action item digest in status created.
pub fn new_fallback_action_item_digest(code: &str, label: &str) -> FallbackActionItemDigest {
    FallbackActionItemDigest {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        rotation_days: 0,
        notes: "".to_string(),
        grace_days: 0,
        status: FallbackActionItemDigestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_action_item_digest("code-735", "label-239");
        assert_eq!(item.status, FallbackActionItemDigestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_fallback_action_item_digest("code-735", "label-239");
        if item.move_to(FallbackActionItemDigestStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackActionItemDigestStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_action_item_digest("code-735", "label-239");
        item.status = FallbackActionItemDigestStatus::Retired;
        assert!(
            item.move_to(FallbackActionItemDigestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
