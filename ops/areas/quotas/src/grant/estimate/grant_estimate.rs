use super::*;

use acme_kit::text;

/// Where a grant estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GrantEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GrantEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant estimate as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantEstimate {
    pub code: String,
    pub label: String,
    pub region: String,
    pub updated_at: i64,
    pub port: i64,
    pub notes: String,
    pub ends_at: i64,
    pub country: String,
    pub score: f64,
    pub source: String,
    pub cost: f64,
    pub status: GrantEstimateStatus,
}

impl GrantEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantEstimateStatus) -> bool {
        match self.status {
            GrantEstimateStatus::Requested => {
                match next {
                    GrantEstimateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantEstimateStatus::Granted => {
                return next == GrantEstimateStatus::Revoked || next == GrantEstimateStatus::Expired;
            }
            GrantEstimateStatus::Revoked => {
                return next == GrantEstimateStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant estimate {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.updated_at as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantEstimate {
        GrantEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            updated_at: self.updated_at,
            port: self.port,
            notes: self.notes.clone(),
            ends_at: self.ends_at,
            country: self.country.clone(),
            score: self.score,
            source: self.source.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a grant estimate in status requested.
pub fn new_grant_estimate(code: &str, label: &str) -> GrantEstimate {
    GrantEstimate {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        updated_at: 0,
        port: 0,
        notes: "".to_string(),
        ends_at: 0,
        country: "".to_string(),
        score: 0.0,
        source: "".to_string(),
        cost: 0.0,
        status: GrantEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_grant_estimate("code-573", "label-764");
        assert_eq!(item.status, GrantEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_grant_estimate("code-573", "label-764");
        if item.move_to(GrantEstimateStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantEstimateStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_grant_estimate("code-573", "label-764");
        item.status = GrantEstimateStatus::Expired;
        assert!(
            item.move_to(GrantEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
