use super::*;

use acme_kit::text;

/// Where a pending delegation mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingDelegationMappingStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingDelegationMappingStatus {
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

impl std::fmt::Display for PendingDelegationMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending delegation mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationMapping {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub notes: String,
    pub channel: String,
    pub duration_minutes: i64,
    pub visible: bool,
    pub expires_at: i64,
    pub maximum: f64,
    pub status: PendingDelegationMappingStatus,
}

impl PendingDelegationMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingDelegationMappingStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingDelegationMappingStatus) -> bool {
        match self.status {
            PendingDelegationMappingStatus::Draft => {
                match next {
                    PendingDelegationMappingStatus::Pending | PendingDelegationMappingStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingDelegationMappingStatus::Pending => {
                match next {
                    PendingDelegationMappingStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingDelegationMappingStatus::Active => {
                return next == PendingDelegationMappingStatus::Suspended
                    || next == PendingDelegationMappingStatus::Closed;
            }
            PendingDelegationMappingStatus::Suspended => {
                return next == PendingDelegationMappingStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingDelegationMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending delegation mapping {} cannot move from {} to {}",
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
        format!("{} {}", "pending delegation mapping", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        (self.maximum * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingDelegationMapping {
        PendingDelegationMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            notes: self.notes.clone(),
            channel: self.channel.clone(),
            duration_minutes: self.duration_minutes,
            visible: self.visible,
            expires_at: self.expires_at,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a pending delegation mapping in status draft.
pub fn new_pending_delegation_mapping(code: &str, label: &str) -> PendingDelegationMapping {
    PendingDelegationMapping {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        notes: "".to_string(),
        channel: "".to_string(),
        duration_minutes: 0,
        visible: false,
        expires_at: 0,
        maximum: 0.0,
        status: PendingDelegationMappingStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_delegation_mapping("code-356", "label-284");
        assert_eq!(item.status, PendingDelegationMappingStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_pending_delegation_mapping("code-356", "label-284");
        if item.move_to(PendingDelegationMappingStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingDelegationMappingStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_delegation_mapping("code-356", "label-284");
        item.status = PendingDelegationMappingStatus::Closed;
        assert!(
            item.move_to(PendingDelegationMappingStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
