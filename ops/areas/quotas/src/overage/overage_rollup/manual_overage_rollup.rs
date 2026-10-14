use super::*;

use acme_kit::text;

/// Where a manual overage rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualOverageRollupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualOverageRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualOverageRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual overage rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualOverageRollup {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub priority: i64,
    pub active: bool,
    pub retries: i64,
    pub segment: String,
    pub issuer: String,
    pub status: ManualOverageRollupStatus,
}

impl ManualOverageRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualOverageRollupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualOverageRollupStatus) -> bool {
        match self.status {
            ManualOverageRollupStatus::Requested => {
                return next == ManualOverageRollupStatus::Expired;
            }
            ManualOverageRollupStatus::Granted => {
                match next {
                    ManualOverageRollupStatus::Revoked | ManualOverageRollupStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualOverageRollupStatus::Revoked => {
                match next {
                    ManualOverageRollupStatus::Expired => {
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

    pub fn move_to(&mut self, next: ManualOverageRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual overage rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.priority as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualOverageRollup {
        ManualOverageRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            priority: self.priority,
            active: self.active,
            retries: self.retries,
            segment: self.segment.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a manual overage rollup in status requested.
pub fn new_manual_overage_rollup(code: &str, title: &str) -> ManualOverageRollup {
    ManualOverageRollup {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        priority: 0,
        active: false,
        retries: 0,
        segment: "".to_string(),
        issuer: "".to_string(),
        status: ManualOverageRollupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_overage_rollup("code-611", "title-236");
        assert_eq!(item.status, ManualOverageRollupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_manual_overage_rollup("code-611", "title-236");
        if item.move_to(ManualOverageRollupStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualOverageRollupStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_overage_rollup("code-611", "title-236");
        item.status = ManualOverageRollupStatus::Expired;
        assert!(
            item.move_to(ManualOverageRollupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
