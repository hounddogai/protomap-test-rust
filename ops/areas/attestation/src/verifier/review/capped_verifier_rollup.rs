use super::*;

use acme_kit::text;

/// Where a capped verifier rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedVerifierRollupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedVerifierRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for CappedVerifierRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped verifier rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierRollup {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub updated_at: i64,
    pub issuer: String,
    pub fingerprint: String,
    pub country: String,
    pub sequence: i64,
    pub ratio: f64,
    pub discount: f64,
    pub cost: f64,
    pub status: CappedVerifierRollupStatus,
}

impl CappedVerifierRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedVerifierRollupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedVerifierRollupStatus) -> bool {
        match self.status {
            CappedVerifierRollupStatus::Planned => {
                return next == CappedVerifierRollupStatus::InProgress || next == CappedVerifierRollupStatus::Cancelled;
            }
            CappedVerifierRollupStatus::Scheduled => {
                match next {
                    CappedVerifierRollupStatus::Completed | CappedVerifierRollupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedVerifierRollupStatus::InProgress => {
                match next {
                    CappedVerifierRollupStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedVerifierRollupStatus::Completed => {
                return next == CappedVerifierRollupStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedVerifierRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped verifier rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped verifier rollup", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.updated_at as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedVerifierRollup {
        CappedVerifierRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            updated_at: self.updated_at,
            issuer: self.issuer.clone(),
            fingerprint: self.fingerprint.clone(),
            country: self.country.clone(),
            sequence: self.sequence,
            ratio: self.ratio,
            discount: self.discount,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a capped verifier rollup in status planned.
pub fn new_capped_verifier_rollup(code: &str, name: &str) -> CappedVerifierRollup {
    CappedVerifierRollup {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        updated_at: 0,
        issuer: "".to_string(),
        fingerprint: "".to_string(),
        country: "".to_string(),
        sequence: 0,
        ratio: 0.0,
        discount: 0.0,
        cost: 0.0,
        status: CappedVerifierRollupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_capped_verifier_rollup("code-243", "name-249");
        assert_eq!(item.status, CappedVerifierRollupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_capped_verifier_rollup("code-243", "name-249");
        if item.move_to(CappedVerifierRollupStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedVerifierRollupStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_capped_verifier_rollup("code-243", "name-249");
        item.status = CappedVerifierRollupStatus::Cancelled;
        assert!(
            item.move_to(CappedVerifierRollupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
