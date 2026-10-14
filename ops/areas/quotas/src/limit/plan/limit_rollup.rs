use super::*;

use acme_kit::text;

/// Where a limit rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LimitRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LimitRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitRollup {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub created_at: i64,
    pub ends_at: i64,
    pub threshold: f64,
    pub quantity: i64,
    pub visible: bool,
    pub cipher: String,
    pub status: LimitRollupStatus,
}

impl LimitRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitRollupStatus) -> bool {
        match self.status {
            LimitRollupStatus::Proposed => {
                match next {
                    LimitRollupStatus::Approved | LimitRollupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitRollupStatus::Approved => {
                return next == LimitRollupStatus::Rejected;
            }
            LimitRollupStatus::Rejected => {
                return next == LimitRollupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.created_at as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitRollup {
        LimitRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            created_at: self.created_at,
            ends_at: self.ends_at,
            threshold: self.threshold,
            quantity: self.quantity,
            visible: self.visible,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a limit rollup in status proposed.
pub fn new_limit_rollup(code: &str, label: &str) -> LimitRollup {
    LimitRollup {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        created_at: 0,
        ends_at: 0,
        threshold: 0.0,
        quantity: 0,
        visible: false,
        cipher: "".to_string(),
        status: LimitRollupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_limit_rollup("code-598", "label-671");
        assert_eq!(item.status, LimitRollupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_limit_rollup("code-598", "label-671");
        if item.move_to(LimitRollupStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitRollupStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_limit_rollup("code-598", "label-671");
        item.status = LimitRollupStatus::Withdrawn;
        assert!(
            item.move_to(LimitRollupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
