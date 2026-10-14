use super::*;

use acme_kit::text;

/// Where a scheduled freeze group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledFreezeGroupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledFreezeGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledFreezeGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled freeze group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeGroup {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub retries: i64,
    pub tenant_id: String,
    pub version: i64,
    pub description: String,
    pub status: ScheduledFreezeGroupStatus,
}

impl ScheduledFreezeGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledFreezeGroupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledFreezeGroupStatus) -> bool {
        match self.status {
            ScheduledFreezeGroupStatus::Requested => {
                return next == ScheduledFreezeGroupStatus::Granted || next == ScheduledFreezeGroupStatus::Revoked;
            }
            ScheduledFreezeGroupStatus::Granted => {
                match next {
                    ScheduledFreezeGroupStatus::Revoked | ScheduledFreezeGroupStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFreezeGroupStatus::Revoked => {
                return next == ScheduledFreezeGroupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledFreezeGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled freeze group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.score - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledFreezeGroup {
        ScheduledFreezeGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            retries: self.retries,
            tenant_id: self.tenant_id.clone(),
            version: self.version,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled freeze group in status requested.
pub fn new_scheduled_freeze_group(code: &str, label: &str) -> ScheduledFreezeGroup {
    ScheduledFreezeGroup {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        retries: 0,
        tenant_id: "".to_string(),
        version: 0,
        description: "".to_string(),
        status: ScheduledFreezeGroupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_freeze_group("code-770", "label-303");
        assert_eq!(item.status, ScheduledFreezeGroupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_scheduled_freeze_group("code-770", "label-303");
        if item.move_to(ScheduledFreezeGroupStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledFreezeGroupStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_freeze_group("code-770", "label-303");
        item.status = ScheduledFreezeGroupStatus::Expired;
        assert!(
            item.move_to(ScheduledFreezeGroupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
