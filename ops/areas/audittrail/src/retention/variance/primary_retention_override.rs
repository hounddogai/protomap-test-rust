use super::*;

use acme_kit::text;

/// Where a primary retention override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRetentionOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryRetentionOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryRetentionOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary retention override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionOverride {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub burst: i64,
    pub approved: bool,
    pub expires_at: i64,
    pub margin: f64,
    pub label: String,
    pub cost: f64,
    pub status: PrimaryRetentionOverrideStatus,
}

impl PrimaryRetentionOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRetentionOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRetentionOverrideStatus) -> bool {
        match self.status {
            PrimaryRetentionOverrideStatus::Requested => {
                match next {
                    PrimaryRetentionOverrideStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRetentionOverrideStatus::Granted => {
                return next == PrimaryRetentionOverrideStatus::Expired;
            }
            PrimaryRetentionOverrideStatus::Revoked => {
                match next {
                    PrimaryRetentionOverrideStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryRetentionOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary retention override {} cannot move from {} to {}",
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
        format!("{} {}", "primary retention override", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        (self.margin * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRetentionOverride {
        PrimaryRetentionOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            burst: self.burst,
            approved: self.approved,
            expires_at: self.expires_at,
            margin: self.margin,
            label: self.label.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary retention override in status requested.
pub fn new_primary_retention_override(code: &str, title: &str) -> PrimaryRetentionOverride {
    PrimaryRetentionOverride {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        burst: 0,
        approved: false,
        expires_at: 0,
        margin: 0.0,
        label: "".to_string(),
        cost: 0.0,
        status: PrimaryRetentionOverrideStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_retention_override("code-420", "title-542");
        assert_eq!(item.status, PrimaryRetentionOverrideStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_primary_retention_override("code-420", "title-542");
        if item.move_to(PrimaryRetentionOverrideStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRetentionOverrideStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_retention_override("code-420", "title-542");
        item.status = PrimaryRetentionOverrideStatus::Expired;
        assert!(
            item.move_to(PrimaryRetentionOverrideStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
