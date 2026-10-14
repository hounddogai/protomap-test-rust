use super::*;

use acme_kit::text;

/// Where a manual sink target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualSinkTargetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualSinkTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualSinkTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual sink target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkTarget {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub updated_at: i64,
    pub rate: f64,
    pub owner_id: String,
    pub segment: String,
    pub margin: f64,
    pub unit: String,
    pub tenant_id: String,
    pub status: ManualSinkTargetStatus,
}

impl ManualSinkTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualSinkTargetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualSinkTargetStatus) -> bool {
        match self.status {
            ManualSinkTargetStatus::Requested => {
                match next {
                    ManualSinkTargetStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSinkTargetStatus::Granted => {
                return next == ManualSinkTargetStatus::Expired;
            }
            ManualSinkTargetStatus::Revoked => {
                return next == ManualSinkTargetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualSinkTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual sink target {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.rate + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualSinkTarget {
        ManualSinkTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            updated_at: self.updated_at,
            rate: self.rate,
            owner_id: self.owner_id.clone(),
            segment: self.segment.clone(),
            margin: self.margin,
            unit: self.unit.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a manual sink target in status requested.
pub fn new_manual_sink_target(code: &str, name: &str) -> ManualSinkTarget {
    ManualSinkTarget {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        updated_at: 0,
        rate: 0.0,
        owner_id: "".to_string(),
        segment: "".to_string(),
        margin: 0.0,
        unit: "".to_string(),
        tenant_id: "".to_string(),
        status: ManualSinkTargetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_sink_target("code-251", "name-426");
        assert_eq!(item.status, ManualSinkTargetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_manual_sink_target("code-251", "name-426");
        if item.move_to(ManualSinkTargetStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualSinkTargetStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_sink_target("code-251", "name-426");
        item.status = ManualSinkTargetStatus::Expired;
        assert!(
            item.move_to(ManualSinkTargetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
