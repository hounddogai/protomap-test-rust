use super::*;

use acme_kit::text;

/// Where a adaptive postmortem digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptivePostmortemDigestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptivePostmortemDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptivePostmortemDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive postmortem digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePostmortemDigest {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub name: String,
    pub reviewer: String,
    pub duration_minutes: i64,
    pub external_ref: String,
    pub status: AdaptivePostmortemDigestStatus,
}

impl AdaptivePostmortemDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptivePostmortemDigestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptivePostmortemDigestStatus) -> bool {
        match self.status {
            AdaptivePostmortemDigestStatus::Requested => {
                return next == AdaptivePostmortemDigestStatus::Revoked;
            }
            AdaptivePostmortemDigestStatus::Granted => {
                match next {
                    AdaptivePostmortemDigestStatus::Revoked | AdaptivePostmortemDigestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptivePostmortemDigestStatus::Revoked => {
                return next == AdaptivePostmortemDigestStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptivePostmortemDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive postmortem digest {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive postmortem digest", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptivePostmortemDigest {
        AdaptivePostmortemDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            duration_minutes: self.duration_minutes,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive postmortem digest in status requested.
pub fn new_adaptive_postmortem_digest(code: &str, label: &str) -> AdaptivePostmortemDigest {
    AdaptivePostmortemDigest {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        name: "".to_string(),
        reviewer: "".to_string(),
        duration_minutes: 0,
        external_ref: "".to_string(),
        status: AdaptivePostmortemDigestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_adaptive_postmortem_digest("code-433", "label-285");
        assert_eq!(item.status, AdaptivePostmortemDigestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_adaptive_postmortem_digest("code-433", "label-285");
        if item.move_to(AdaptivePostmortemDigestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptivePostmortemDigestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_adaptive_postmortem_digest("code-433", "label-285");
        item.status = AdaptivePostmortemDigestStatus::Expired;
        assert!(
            item.move_to(AdaptivePostmortemDigestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
