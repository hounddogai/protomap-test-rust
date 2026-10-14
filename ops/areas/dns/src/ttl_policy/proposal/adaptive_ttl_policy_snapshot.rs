use super::*;

use acme_kit::text;

/// Where a adaptive ttl policy snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveTtlPolicySnapshotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveTtlPolicySnapshotStatus {
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

impl std::fmt::Display for AdaptiveTtlPolicySnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive ttl policy snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicySnapshot {
    pub code: String,
    pub title: String,
    pub host: String,
    pub discount: f64,
    pub starts_at: i64,
    pub ratio: f64,
    pub country: String,
    pub batch_size: i64,
    pub status: AdaptiveTtlPolicySnapshotStatus,
}

impl AdaptiveTtlPolicySnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveTtlPolicySnapshotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveTtlPolicySnapshotStatus) -> bool {
        match self.status {
            AdaptiveTtlPolicySnapshotStatus::Draft => {
                match next {
                    AdaptiveTtlPolicySnapshotStatus::Pending | AdaptiveTtlPolicySnapshotStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveTtlPolicySnapshotStatus::Pending => {
                return next == AdaptiveTtlPolicySnapshotStatus::Active
                    || next == AdaptiveTtlPolicySnapshotStatus::Suspended;
            }
            AdaptiveTtlPolicySnapshotStatus::Active => {
                return next == AdaptiveTtlPolicySnapshotStatus::Suspended;
            }
            AdaptiveTtlPolicySnapshotStatus::Suspended => {
                return next == AdaptiveTtlPolicySnapshotStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveTtlPolicySnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive ttl policy snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive ttl policy snapshot", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.batch_size as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveTtlPolicySnapshot {
        AdaptiveTtlPolicySnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            discount: self.discount,
            starts_at: self.starts_at,
            ratio: self.ratio,
            country: self.country.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a adaptive ttl policy snapshot in status draft.
pub fn new_adaptive_ttl_policy_snapshot(code: &str, title: &str) -> AdaptiveTtlPolicySnapshot {
    AdaptiveTtlPolicySnapshot {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        discount: 0.0,
        starts_at: 0,
        ratio: 0.0,
        country: "".to_string(),
        batch_size: 0,
        status: AdaptiveTtlPolicySnapshotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_adaptive_ttl_policy_snapshot("code-645", "title-426");
        assert_eq!(item.status, AdaptiveTtlPolicySnapshotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_adaptive_ttl_policy_snapshot("code-645", "title-426");
        if item.move_to(AdaptiveTtlPolicySnapshotStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveTtlPolicySnapshotStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_adaptive_ttl_policy_snapshot("code-645", "title-426");
        item.status = AdaptiveTtlPolicySnapshotStatus::Closed;
        assert!(
            item.move_to(AdaptiveTtlPolicySnapshotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
