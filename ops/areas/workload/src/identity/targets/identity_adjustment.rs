use super::*;

use acme_kit::text;

/// Where a identity adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityAdjustmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl IdentityAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for IdentityAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAdjustment {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub reviewer: String,
    pub score: f64,
    pub ends_at: i64,
    pub locale: String,
    pub ceiling: f64,
    pub cipher: String,
    pub rotation_days: i64,
    pub status: IdentityAdjustmentStatus,
}

impl IdentityAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityAdjustmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityAdjustmentStatus) -> bool {
        match self.status {
            IdentityAdjustmentStatus::Proposed => {
                return next == IdentityAdjustmentStatus::Withdrawn;
            }
            IdentityAdjustmentStatus::Approved => {
                return next == IdentityAdjustmentStatus::Rejected;
            }
            IdentityAdjustmentStatus::Rejected => {
                match next {
                    IdentityAdjustmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: IdentityAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity adjustment {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.window_days as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityAdjustment {
        IdentityAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            reviewer: self.reviewer.clone(),
            score: self.score,
            ends_at: self.ends_at,
            locale: self.locale.clone(),
            ceiling: self.ceiling,
            cipher: self.cipher.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a identity adjustment in status proposed.
pub fn new_identity_adjustment(code: &str, label: &str) -> IdentityAdjustment {
    IdentityAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        reviewer: "".to_string(),
        score: 0.0,
        ends_at: 0,
        locale: "".to_string(),
        ceiling: 0.0,
        cipher: "".to_string(),
        rotation_days: 0,
        status: IdentityAdjustmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_identity_adjustment("code-598", "label-197");
        assert_eq!(item.status, IdentityAdjustmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_identity_adjustment("code-598", "label-197");
        if item.move_to(IdentityAdjustmentStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityAdjustmentStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_identity_adjustment("code-598", "label-197");
        item.status = IdentityAdjustmentStatus::Withdrawn;
        assert!(
            item.move_to(IdentityAdjustmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
