use super::*;

use acme_kit::text;

/// Where a scheduled federation request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledFederationRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledFederationRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledFederationRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled federation request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationRequest {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub archived: bool,
    pub priority: i64,
    pub status: ScheduledFederationRequestStatus,
}

impl ScheduledFederationRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledFederationRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledFederationRequestStatus) -> bool {
        match self.status {
            ScheduledFederationRequestStatus::Requested => {
                return next == ScheduledFederationRequestStatus::Granted
                    || next == ScheduledFederationRequestStatus::Expired;
            }
            ScheduledFederationRequestStatus::Granted => {
                match next {
                    ScheduledFederationRequestStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFederationRequestStatus::Revoked => {
                match next {
                    ScheduledFederationRequestStatus::Expired => {
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

    pub fn move_to(&mut self, next: ScheduledFederationRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled federation request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled federation request", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> ScheduledFederationRequest {
        ScheduledFederationRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            archived: self.archived,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a scheduled federation request in status requested.
pub fn new_scheduled_federation_request(code: &str, title: &str) -> ScheduledFederationRequest {
    ScheduledFederationRequest {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        archived: false,
        priority: 0,
        status: ScheduledFederationRequestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_federation_request("code-23", "title-826");
        assert_eq!(item.status, ScheduledFederationRequestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_scheduled_federation_request("code-23", "title-826");
        if item.move_to(ScheduledFederationRequestStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledFederationRequestStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_federation_request("code-23", "title-826");
        item.status = ScheduledFederationRequestStatus::Expired;
        assert!(
            item.move_to(ScheduledFederationRequestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
