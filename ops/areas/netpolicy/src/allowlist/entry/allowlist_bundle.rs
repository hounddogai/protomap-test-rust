use super::*;

use acme_kit::text;

/// Where a allowlist bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistBundleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AllowlistBundleStatus {
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

impl std::fmt::Display for AllowlistBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One allowlist bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBundle {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub ends_at: i64,
    pub fingerprint: String,
    pub tenant_id: String,
    pub cost: f64,
    pub port: i64,
    pub status: AllowlistBundleStatus,
}

impl AllowlistBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistBundleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistBundleStatus) -> bool {
        match self.status {
            AllowlistBundleStatus::Draft => {
                match next {
                    AllowlistBundleStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistBundleStatus::Pending => {
                match next {
                    AllowlistBundleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistBundleStatus::Active => {
                match next {
                    AllowlistBundleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistBundleStatus::Suspended => {
                match next {
                    AllowlistBundleStatus::Closed => {
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

    pub fn move_to(&mut self, next: AllowlistBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist bundle {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.port as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistBundle {
        AllowlistBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            ends_at: self.ends_at,
            fingerprint: self.fingerprint.clone(),
            tenant_id: self.tenant_id.clone(),
            cost: self.cost,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a allowlist bundle in status draft.
pub fn new_allowlist_bundle(code: &str, label: &str) -> AllowlistBundle {
    AllowlistBundle {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        ends_at: 0,
        fingerprint: "".to_string(),
        tenant_id: "".to_string(),
        cost: 0.0,
        port: 0,
        status: AllowlistBundleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_allowlist_bundle("code-593", "label-671");
        assert_eq!(item.status, AllowlistBundleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_allowlist_bundle("code-593", "label-671");
        if item.move_to(AllowlistBundleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistBundleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_allowlist_bundle("code-593", "label-671");
        item.status = AllowlistBundleStatus::Closed;
        assert!(
            item.move_to(AllowlistBundleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
