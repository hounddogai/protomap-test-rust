use super::*;

use acme_kit::text;

/// Where a manual allowlist bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualAllowlistBundleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualAllowlistBundleStatus {
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

impl std::fmt::Display for ManualAllowlistBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual allowlist bundle as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistBundle {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub visible: bool,
    pub cost: f64,
    pub timezone: String,
    pub ends_at: i64,
    pub created_at: i64,
    pub amount: f64,
    pub window_days: i64,
    pub status: ManualAllowlistBundleStatus,
}

impl ManualAllowlistBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualAllowlistBundleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualAllowlistBundleStatus) -> bool {
        match self.status {
            ManualAllowlistBundleStatus::Draft => {
                return next == ManualAllowlistBundleStatus::Suspended;
            }
            ManualAllowlistBundleStatus::Pending => {
                return next == ManualAllowlistBundleStatus::Suspended;
            }
            ManualAllowlistBundleStatus::Active => {
                match next {
                    ManualAllowlistBundleStatus::Suspended | ManualAllowlistBundleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualAllowlistBundleStatus::Suspended => {
                match next {
                    ManualAllowlistBundleStatus::Closed => {
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

    pub fn move_to(&mut self, next: ManualAllowlistBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual allowlist bundle {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.cost + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualAllowlistBundle {
        ManualAllowlistBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            visible: self.visible,
            cost: self.cost,
            timezone: self.timezone.clone(),
            ends_at: self.ends_at,
            created_at: self.created_at,
            amount: self.amount,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a manual allowlist bundle in status draft.
pub fn new_manual_allowlist_bundle(code: &str, label: &str) -> ManualAllowlistBundle {
    ManualAllowlistBundle {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        visible: false,
        cost: 0.0,
        timezone: "".to_string(),
        ends_at: 0,
        created_at: 0,
        amount: 0.0,
        window_days: 0,
        status: ManualAllowlistBundleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_manual_allowlist_bundle("code-466", "label-730");
        assert_eq!(item.status, ManualAllowlistBundleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_manual_allowlist_bundle("code-466", "label-730");
        if item.move_to(ManualAllowlistBundleStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualAllowlistBundleStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_manual_allowlist_bundle("code-466", "label-730");
        item.status = ManualAllowlistBundleStatus::Closed;
        assert!(
            item.move_to(ManualAllowlistBundleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
