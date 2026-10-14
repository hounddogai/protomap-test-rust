use super::*;

use acme_kit::text;

/// Where a pending overage bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingOverageBundleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingOverageBundleStatus {
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

impl std::fmt::Display for PendingOverageBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending overage bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingOverageBundle {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub port: i64,
    pub sequence: i64,
    pub window_days: i64,
    pub status: PendingOverageBundleStatus,
}

impl PendingOverageBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingOverageBundleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingOverageBundleStatus) -> bool {
        match self.status {
            PendingOverageBundleStatus::Draft => {
                match next {
                    PendingOverageBundleStatus::Pending | PendingOverageBundleStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingOverageBundleStatus::Pending => {
                match next {
                    PendingOverageBundleStatus::Suspended | PendingOverageBundleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingOverageBundleStatus::Active => {
                return next == PendingOverageBundleStatus::Closed;
            }
            PendingOverageBundleStatus::Suspended => {
                match next {
                    PendingOverageBundleStatus::Closed => {
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

    pub fn move_to(&mut self, next: PendingOverageBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending overage bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.port as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingOverageBundle {
        PendingOverageBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            port: self.port,
            sequence: self.sequence,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending overage bundle in status draft.
pub fn new_pending_overage_bundle(code: &str, label: &str) -> PendingOverageBundle {
    PendingOverageBundle {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        port: 0,
        sequence: 0,
        window_days: 0,
        status: PendingOverageBundleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_overage_bundle("code-349", "label-813");
        assert_eq!(item.status, PendingOverageBundleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_pending_overage_bundle("code-349", "label-813");
        if item.move_to(PendingOverageBundleStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingOverageBundleStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_overage_bundle("code-349", "label-813");
        item.status = PendingOverageBundleStatus::Closed;
        assert!(
            item.move_to(PendingOverageBundleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
