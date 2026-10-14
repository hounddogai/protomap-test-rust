use super::*;

use acme_kit::text;

/// Where a rolling limit mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingLimitMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingLimitMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingLimitMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling limit mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitMapping {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub quota_units: i64,
    pub burst: i64,
    pub principal: String,
    pub quantity: i64,
    pub cipher: String,
    pub updated_at: i64,
    pub host: String,
    pub status: RollingLimitMappingStatus,
}

impl RollingLimitMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingLimitMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingLimitMappingStatus) -> bool {
        match self.status {
            RollingLimitMappingStatus::Proposed => {
                return next == RollingLimitMappingStatus::Approved;
            }
            RollingLimitMappingStatus::Approved => {
                return next == RollingLimitMappingStatus::Rejected || next == RollingLimitMappingStatus::Withdrawn;
            }
            RollingLimitMappingStatus::Rejected => {
                match next {
                    RollingLimitMappingStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RollingLimitMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling limit mapping {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.sequence as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingLimitMapping {
        RollingLimitMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            quota_units: self.quota_units,
            burst: self.burst,
            principal: self.principal.clone(),
            quantity: self.quantity,
            cipher: self.cipher.clone(),
            updated_at: self.updated_at,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling limit mapping in status proposed.
pub fn new_rolling_limit_mapping(code: &str, title: &str) -> RollingLimitMapping {
    RollingLimitMapping {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        quota_units: 0,
        burst: 0,
        principal: "".to_string(),
        quantity: 0,
        cipher: "".to_string(),
        updated_at: 0,
        host: "".to_string(),
        status: RollingLimitMappingStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_limit_mapping("code-95", "title-395");
        assert_eq!(item.status, RollingLimitMappingStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rolling_limit_mapping("code-95", "title-395");
        if item.move_to(RollingLimitMappingStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingLimitMappingStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_limit_mapping("code-95", "title-395");
        item.status = RollingLimitMappingStatus::Withdrawn;
        assert!(
            item.move_to(RollingLimitMappingStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
