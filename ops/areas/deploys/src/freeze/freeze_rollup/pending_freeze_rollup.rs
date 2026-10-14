use super::*;

use acme_kit::text;

/// Where a pending freeze rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingFreezeRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingFreezeRollupStatus {
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

impl std::fmt::Display for PendingFreezeRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending freeze rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeRollup {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub grace_days: i64,
    pub maximum: f64,
    pub checksum: String,
    pub principal: String,
    pub version: i64,
    pub margin: f64,
    pub ends_at: i64,
    pub region: String,
    pub status: PendingFreezeRollupStatus,
}

impl PendingFreezeRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingFreezeRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingFreezeRollupStatus) -> bool {
        match self.status {
            PendingFreezeRollupStatus::Draft => {
                return next == PendingFreezeRollupStatus::Closed;
            }
            PendingFreezeRollupStatus::Pending => {
                match next {
                    PendingFreezeRollupStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingFreezeRollupStatus::Active => {
                match next {
                    PendingFreezeRollupStatus::Suspended | PendingFreezeRollupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingFreezeRollupStatus::Suspended => {
                match next {
                    PendingFreezeRollupStatus::Closed => {
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

    pub fn move_to(&mut self, next: PendingFreezeRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending freeze rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.version as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingFreezeRollup {
        PendingFreezeRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            grace_days: self.grace_days,
            maximum: self.maximum,
            checksum: self.checksum.clone(),
            principal: self.principal.clone(),
            version: self.version,
            margin: self.margin,
            ends_at: self.ends_at,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a pending freeze rollup in status draft.
pub fn new_pending_freeze_rollup(code: &str, label: &str) -> PendingFreezeRollup {
    PendingFreezeRollup {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        grace_days: 0,
        maximum: 0.0,
        checksum: "".to_string(),
        principal: "".to_string(),
        version: 0,
        margin: 0.0,
        ends_at: 0,
        region: "".to_string(),
        status: PendingFreezeRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_freeze_rollup("code-86", "label-240");
        assert_eq!(item.status, PendingFreezeRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_pending_freeze_rollup("code-86", "label-240");
        if item.move_to(PendingFreezeRollupStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingFreezeRollupStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_freeze_rollup("code-86", "label-240");
        item.status = PendingFreezeRollupStatus::Closed;
        assert!(
            item.move_to(PendingFreezeRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
