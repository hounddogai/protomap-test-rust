use super::*;

use acme_kit::text;

/// Where a adaptive slo tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveSloTierStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptiveSloTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptiveSloTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive slo tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloTier {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub host: String,
    pub checksum: String,
    pub name: String,
    pub namespace: String,
    pub rotation_days: i64,
    pub status: AdaptiveSloTierStatus,
}

impl AdaptiveSloTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveSloTierStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveSloTierStatus) -> bool {
        match self.status {
            AdaptiveSloTierStatus::Requested => {
                match next {
                    AdaptiveSloTierStatus::Revoked | AdaptiveSloTierStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSloTierStatus::Granted => {
                match next {
                    AdaptiveSloTierStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSloTierStatus::Revoked => {
                return next == AdaptiveSloTierStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveSloTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive slo tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.maximum + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveSloTier {
        AdaptiveSloTier {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            host: self.host.clone(),
            checksum: self.checksum.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a adaptive slo tier in status requested.
pub fn new_adaptive_slo_tier(code: &str, title: &str) -> AdaptiveSloTier {
    AdaptiveSloTier {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        host: "".to_string(),
        checksum: "".to_string(),
        name: "".to_string(),
        namespace: "".to_string(),
        rotation_days: 0,
        status: AdaptiveSloTierStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_adaptive_slo_tier("code-449", "title-600");
        assert_eq!(item.status, AdaptiveSloTierStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_adaptive_slo_tier("code-449", "title-600");
        if item.move_to(AdaptiveSloTierStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveSloTierStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_adaptive_slo_tier("code-449", "title-600");
        item.status = AdaptiveSloTierStatus::Expired;
        assert!(
            item.move_to(AdaptiveSloTierStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
