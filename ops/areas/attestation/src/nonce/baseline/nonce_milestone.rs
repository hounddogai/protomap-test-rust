use super::*;

use acme_kit::text;

/// Where a nonce milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceMilestoneStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl NonceMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for NonceMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce milestone as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceMilestone {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub currency: String,
    pub region: String,
    pub name: String,
    pub ends_at: i64,
    pub status: NonceMilestoneStatus,
}

impl NonceMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceMilestoneStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceMilestoneStatus) -> bool {
        match self.status {
            NonceMilestoneStatus::Requested => {
                match next {
                    NonceMilestoneStatus::Granted | NonceMilestoneStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceMilestoneStatus::Granted => {
                return next == NonceMilestoneStatus::Expired;
            }
            NonceMilestoneStatus::Revoked => {
                match next {
                    NonceMilestoneStatus::Expired => {
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

    pub fn move_to(&mut self, next: NonceMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "nonce milestone", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.ends_at as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceMilestone {
        NonceMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            currency: self.currency.clone(),
            region: self.region.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a nonce milestone in status requested.
pub fn new_nonce_milestone(code: &str, title: &str) -> NonceMilestone {
    NonceMilestone {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        currency: "".to_string(),
        region: "".to_string(),
        name: "".to_string(),
        ends_at: 0,
        status: NonceMilestoneStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_nonce_milestone("code-586", "title-323");
        assert_eq!(item.status, NonceMilestoneStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_nonce_milestone("code-586", "title-323");
        if item.move_to(NonceMilestoneStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceMilestoneStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_nonce_milestone("code-586", "title-323");
        item.status = NonceMilestoneStatus::Expired;
        assert!(
            item.move_to(NonceMilestoneStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
