use super::*;

use acme_kit::text;

/// Where a seasonal silence target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalSilenceTargetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalSilenceTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalSilenceTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal silence target as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSilenceTarget {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub cluster: String,
    pub minimum: f64,
    pub approved: bool,
    pub batch_size: i64,
    pub namespace: String,
    pub status: SeasonalSilenceTargetStatus,
}

impl SeasonalSilenceTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalSilenceTargetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalSilenceTargetStatus) -> bool {
        match self.status {
            SeasonalSilenceTargetStatus::Requested => {
                match next {
                    SeasonalSilenceTargetStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalSilenceTargetStatus::Granted => {
                match next {
                    SeasonalSilenceTargetStatus::Revoked | SeasonalSilenceTargetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalSilenceTargetStatus::Revoked => {
                return next == SeasonalSilenceTargetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalSilenceTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal silence target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal silence target", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.batch_size as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalSilenceTarget {
        SeasonalSilenceTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            cluster: self.cluster.clone(),
            minimum: self.minimum,
            approved: self.approved,
            batch_size: self.batch_size,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a seasonal silence target in status requested.
pub fn new_seasonal_silence_target(code: &str, name: &str) -> SeasonalSilenceTarget {
    SeasonalSilenceTarget {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        cluster: "".to_string(),
        minimum: 0.0,
        approved: false,
        batch_size: 0,
        namespace: "".to_string(),
        status: SeasonalSilenceTargetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_silence_target("code-645", "name-102");
        assert_eq!(item.status, SeasonalSilenceTargetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_seasonal_silence_target("code-645", "name-102");
        if item.move_to(SeasonalSilenceTargetStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalSilenceTargetStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_silence_target("code-645", "name-102");
        item.status = SeasonalSilenceTargetStatus::Expired;
        assert!(
            item.move_to(SeasonalSilenceTargetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
