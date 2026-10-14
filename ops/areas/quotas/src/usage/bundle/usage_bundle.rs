use super::*;

use acme_kit::text;

/// Where a usage bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl UsageBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for UsageBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a usage bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageBundle {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub external_ref: String,
    pub tenant_id: String,
    pub region: String,
    pub quantity: i64,
    pub starts_at: i64,
    pub status: UsageBundleStatus,
}

impl UsageBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageBundleStatus) -> bool {
        match self.status {
            UsageBundleStatus::Open => {
                match next {
                    UsageBundleStatus::Resolved | UsageBundleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageBundleStatus::InReview => {
                match next {
                    UsageBundleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageBundleStatus::Resolved => {
                return next == UsageBundleStatus::Archived;
            }
            UsageBundleStatus::Reopened => {
                match next {
                    UsageBundleStatus::Archived => {
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

    pub fn move_to(&mut self, next: UsageBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage bundle {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.quantity as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageBundle {
        UsageBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            external_ref: self.external_ref.clone(),
            tenant_id: self.tenant_id.clone(),
            region: self.region.clone(),
            quantity: self.quantity,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a usage bundle in status open.
pub fn new_usage_bundle(code: &str, name: &str) -> UsageBundle {
    UsageBundle {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        external_ref: "".to_string(),
        tenant_id: "".to_string(),
        region: "".to_string(),
        quantity: 0,
        starts_at: 0,
        status: UsageBundleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_usage_bundle("code-817", "name-115");
        assert_eq!(item.status, UsageBundleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_usage_bundle("code-817", "name-115");
        if item.move_to(UsageBundleStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageBundleStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_usage_bundle("code-817", "name-115");
        item.status = UsageBundleStatus::Archived;
        assert!(item.move_to(UsageBundleStatus::Open).is_err(), "expected move to fail");
    }
}
