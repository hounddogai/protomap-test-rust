use super::*;

use acme_kit::text;

/// Where a delegation baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationBaselineStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DelegationBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DelegationBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBaseline {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub name: String,
    pub weight: f64,
    pub grace_days: i64,
    pub expires_at: i64,
    pub host: String,
    pub status: DelegationBaselineStatus,
}

impl DelegationBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationBaselineStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationBaselineStatus) -> bool {
        match self.status {
            DelegationBaselineStatus::Proposed => {
                match next {
                    DelegationBaselineStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationBaselineStatus::Approved => {
                return next == DelegationBaselineStatus::Rejected || next == DelegationBaselineStatus::Withdrawn;
            }
            DelegationBaselineStatus::Rejected => {
                match next {
                    DelegationBaselineStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DelegationBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.weight * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationBaseline {
        DelegationBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            name: self.name.clone(),
            weight: self.weight,
            grace_days: self.grace_days,
            expires_at: self.expires_at,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a delegation baseline in status proposed.
pub fn new_delegation_baseline(code: &str, label: &str) -> DelegationBaseline {
    DelegationBaseline {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        name: "".to_string(),
        weight: 0.0,
        grace_days: 0,
        expires_at: 0,
        host: "".to_string(),
        status: DelegationBaselineStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_delegation_baseline("code-542", "label-255");
        assert_eq!(item.status, DelegationBaselineStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_delegation_baseline("code-542", "label-255");
        if item.move_to(DelegationBaselineStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationBaselineStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_delegation_baseline("code-542", "label-255");
        item.status = DelegationBaselineStatus::Withdrawn;
        assert!(
            item.move_to(DelegationBaselineStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
