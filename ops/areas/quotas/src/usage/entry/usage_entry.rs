use super::*;

use acme_kit::text;

/// Where a usage entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl UsageEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for UsageEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageEntry {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub external_ref: String,
    pub country: String,
    pub tenant_id: String,
    pub fingerprint: String,
    pub principal: String,
    pub visible: bool,
    pub channel: String,
    pub locale: String,
    pub status: UsageEntryStatus,
}

impl UsageEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageEntryStatus) -> bool {
        match self.status {
            UsageEntryStatus::Proposed => {
                match next {
                    UsageEntryStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageEntryStatus::Approved => {
                match next {
                    UsageEntryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageEntryStatus::Rejected => {
                return next == UsageEntryStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "usage entry", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> UsageEntry {
        UsageEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            external_ref: self.external_ref.clone(),
            country: self.country.clone(),
            tenant_id: self.tenant_id.clone(),
            fingerprint: self.fingerprint.clone(),
            principal: self.principal.clone(),
            visible: self.visible,
            channel: self.channel.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a usage entry in status proposed.
pub fn new_usage_entry(code: &str, name: &str) -> UsageEntry {
    UsageEntry {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        external_ref: "".to_string(),
        country: "".to_string(),
        tenant_id: "".to_string(),
        fingerprint: "".to_string(),
        principal: "".to_string(),
        visible: false,
        channel: "".to_string(),
        locale: "".to_string(),
        status: UsageEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_usage_entry("code-871", "name-770");
        assert_eq!(item.status, UsageEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_usage_entry("code-871", "name-770");
        if item.move_to(UsageEntryStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageEntryStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_usage_entry("code-871", "name-770");
        item.status = UsageEntryStatus::Withdrawn;
        assert!(
            item.move_to(UsageEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
