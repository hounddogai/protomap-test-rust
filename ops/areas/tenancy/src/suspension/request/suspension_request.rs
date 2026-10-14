use super::*;

use acme_kit::text;

/// Where a suspension request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SuspensionRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SuspensionRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRequest {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub port: i64,
    pub host: String,
    pub rate: f64,
    pub fingerprint: String,
    pub budget: f64,
    pub maximum: f64,
    pub status: SuspensionRequestStatus,
}

impl SuspensionRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionRequestStatus) -> bool {
        match self.status {
            SuspensionRequestStatus::Requested => {
                match next {
                    SuspensionRequestStatus::Revoked | SuspensionRequestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionRequestStatus::Granted => {
                match next {
                    SuspensionRequestStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionRequestStatus::Revoked => {
                match next {
                    SuspensionRequestStatus::Expired => {
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

    pub fn move_to(&mut self, next: SuspensionRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "suspension request", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.port as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionRequest {
        SuspensionRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            port: self.port,
            host: self.host.clone(),
            rate: self.rate,
            fingerprint: self.fingerprint.clone(),
            budget: self.budget,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a suspension request in status requested.
pub fn new_suspension_request(code: &str, label: &str) -> SuspensionRequest {
    SuspensionRequest {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        port: 0,
        host: "".to_string(),
        rate: 0.0,
        fingerprint: "".to_string(),
        budget: 0.0,
        maximum: 0.0,
        status: SuspensionRequestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_suspension_request("code-830", "label-980");
        assert_eq!(item.status, SuspensionRequestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_suspension_request("code-830", "label-980");
        if item.move_to(SuspensionRequestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionRequestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_suspension_request("code-830", "label-980");
        item.status = SuspensionRequestStatus::Expired;
        assert!(
            item.move_to(SuspensionRequestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
