use super::*;

use acme_kit::text;

/// Where a peering rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringRollupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PeeringRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PeeringRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRollup {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub score: f64,
    pub source: String,
    pub tenant_id: String,
    pub status: PeeringRollupStatus,
}

impl PeeringRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringRollupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringRollupStatus) -> bool {
        match self.status {
            PeeringRollupStatus::Requested => {
                return next == PeeringRollupStatus::Granted || next == PeeringRollupStatus::Expired;
            }
            PeeringRollupStatus::Granted => {
                match next {
                    PeeringRollupStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringRollupStatus::Revoked => {
                return next == PeeringRollupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering rollup {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "peering rollup", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.score - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringRollup {
        PeeringRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            score: self.score,
            source: self.source.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a peering rollup in status requested.
pub fn new_peering_rollup(code: &str, name: &str) -> PeeringRollup {
    PeeringRollup {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        score: 0.0,
        source: "".to_string(),
        tenant_id: "".to_string(),
        status: PeeringRollupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_peering_rollup("code-531", "name-753");
        assert_eq!(item.status, PeeringRollupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_peering_rollup("code-531", "name-753");
        if item.move_to(PeeringRollupStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringRollupStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_peering_rollup("code-531", "name-753");
        item.status = PeeringRollupStatus::Expired;
        assert!(
            item.move_to(PeeringRollupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
