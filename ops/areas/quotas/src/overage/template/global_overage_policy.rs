use super::*;

use acme_kit::text;

/// Where a global overage policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalOveragePolicyStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalOveragePolicyStatus {
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

impl std::fmt::Display for GlobalOveragePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global overage policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOveragePolicy {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub tenant_id: String,
    pub issuer: String,
    pub host: String,
    pub archived: bool,
    pub floor_value: f64,
    pub sequence: i64,
    pub scope: String,
    pub status: GlobalOveragePolicyStatus,
}

impl GlobalOveragePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalOveragePolicyStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalOveragePolicyStatus) -> bool {
        match self.status {
            GlobalOveragePolicyStatus::Draft => {
                match next {
                    GlobalOveragePolicyStatus::Pending | GlobalOveragePolicyStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalOveragePolicyStatus::Pending => {
                return next == GlobalOveragePolicyStatus::Active;
            }
            GlobalOveragePolicyStatus::Active => {
                match next {
                    GlobalOveragePolicyStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalOveragePolicyStatus::Suspended => {
                return next == GlobalOveragePolicyStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalOveragePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global overage policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.sequence as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalOveragePolicy {
        GlobalOveragePolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            tenant_id: self.tenant_id.clone(),
            issuer: self.issuer.clone(),
            host: self.host.clone(),
            archived: self.archived,
            floor_value: self.floor_value,
            sequence: self.sequence,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a global overage policy in status draft.
pub fn new_global_overage_policy(code: &str, name: &str) -> GlobalOveragePolicy {
    GlobalOveragePolicy {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        tenant_id: "".to_string(),
        issuer: "".to_string(),
        host: "".to_string(),
        archived: false,
        floor_value: 0.0,
        sequence: 0,
        scope: "".to_string(),
        status: GlobalOveragePolicyStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_overage_policy("code-415", "name-414");
        assert_eq!(item.status, GlobalOveragePolicyStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_global_overage_policy("code-415", "name-414");
        if item.move_to(GlobalOveragePolicyStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalOveragePolicyStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_overage_policy("code-415", "name-414");
        item.status = GlobalOveragePolicyStatus::Closed;
        assert!(
            item.move_to(GlobalOveragePolicyStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
