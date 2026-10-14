use super::*;

use acme_kit::text;

/// Where a global sink bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSinkBundleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalSinkBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalSinkBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global sink bundle as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkBundle {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub updated_at: i64,
    pub cost: f64,
    pub budget: f64,
    pub scope: String,
    pub locale: String,
    pub region: String,
    pub status: GlobalSinkBundleStatus,
}

impl GlobalSinkBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSinkBundleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSinkBundleStatus) -> bool {
        match self.status {
            GlobalSinkBundleStatus::Requested => {
                match next {
                    GlobalSinkBundleStatus::Revoked | GlobalSinkBundleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSinkBundleStatus::Granted => {
                return next == GlobalSinkBundleStatus::Revoked || next == GlobalSinkBundleStatus::Expired;
            }
            GlobalSinkBundleStatus::Revoked => {
                return next == GlobalSinkBundleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalSinkBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global sink bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.cost + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalSinkBundle {
        GlobalSinkBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            updated_at: self.updated_at,
            cost: self.cost,
            budget: self.budget,
            scope: self.scope.clone(),
            locale: self.locale.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a global sink bundle in status requested.
pub fn new_global_sink_bundle(code: &str, name: &str) -> GlobalSinkBundle {
    GlobalSinkBundle {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        updated_at: 0,
        cost: 0.0,
        budget: 0.0,
        scope: "".to_string(),
        locale: "".to_string(),
        region: "".to_string(),
        status: GlobalSinkBundleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_global_sink_bundle("code-562", "name-178");
        assert_eq!(item.status, GlobalSinkBundleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_global_sink_bundle("code-562", "name-178");
        if item.move_to(GlobalSinkBundleStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalSinkBundleStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_global_sink_bundle("code-562", "name-178");
        item.status = GlobalSinkBundleStatus::Expired;
        assert!(
            item.move_to(GlobalSinkBundleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
