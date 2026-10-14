use super::*;

use acme_kit::text;

/// Where a primary query profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryQueryProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryQueryProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryQueryProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary query profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQueryProfile {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub retries: i64,
    pub visible: bool,
    pub amount: f64,
    pub minimum: f64,
    pub status: PrimaryQueryProfileStatus,
}

impl PrimaryQueryProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryQueryProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryQueryProfileStatus) -> bool {
        match self.status {
            PrimaryQueryProfileStatus::Requested => {
                return next == PrimaryQueryProfileStatus::Expired;
            }
            PrimaryQueryProfileStatus::Granted => {
                match next {
                    PrimaryQueryProfileStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryQueryProfileStatus::Revoked => {
                match next {
                    PrimaryQueryProfileStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryQueryProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary query profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.amount * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryQueryProfile {
        PrimaryQueryProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            retries: self.retries,
            visible: self.visible,
            amount: self.amount,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary query profile in status requested.
pub fn new_primary_query_profile(code: &str, title: &str) -> PrimaryQueryProfile {
    PrimaryQueryProfile {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        retries: 0,
        visible: false,
        amount: 0.0,
        minimum: 0.0,
        status: PrimaryQueryProfileStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_query_profile("code-337", "title-318");
        assert_eq!(item.status, PrimaryQueryProfileStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_primary_query_profile("code-337", "title-318");
        if item.move_to(PrimaryQueryProfileStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryQueryProfileStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_query_profile("code-337", "title-318");
        item.status = PrimaryQueryProfileStatus::Expired;
        assert!(
            item.move_to(PrimaryQueryProfileStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
