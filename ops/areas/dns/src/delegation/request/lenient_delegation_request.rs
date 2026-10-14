use super::*;

use acme_kit::text;

/// Where a lenient delegation request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientDelegationRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientDelegationRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for LenientDelegationRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient delegation request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationRequest {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub priority: i64,
    pub minimum: f64,
    pub starts_at: i64,
    pub channel: String,
    pub audience: String,
    pub maximum: f64,
    pub title: String,
    pub status: LenientDelegationRequestStatus,
}

impl LenientDelegationRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientDelegationRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientDelegationRequestStatus) -> bool {
        match self.status {
            LenientDelegationRequestStatus::Draft => {
                match next {
                    LenientDelegationRequestStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientDelegationRequestStatus::Pending => {
                return next == LenientDelegationRequestStatus::Suspended;
            }
            LenientDelegationRequestStatus::Active => {
                match next {
                    LenientDelegationRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientDelegationRequestStatus::Suspended => {
                match next {
                    LenientDelegationRequestStatus::Closed => {
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

    pub fn move_to(&mut self, next: LenientDelegationRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient delegation request {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient delegation request", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.starts_at as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientDelegationRequest {
        LenientDelegationRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            priority: self.priority,
            minimum: self.minimum,
            starts_at: self.starts_at,
            channel: self.channel.clone(),
            audience: self.audience.clone(),
            maximum: self.maximum,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a lenient delegation request in status draft.
pub fn new_lenient_delegation_request(code: &str, label: &str) -> LenientDelegationRequest {
    LenientDelegationRequest {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        priority: 0,
        minimum: 0.0,
        starts_at: 0,
        channel: "".to_string(),
        audience: "".to_string(),
        maximum: 0.0,
        title: "".to_string(),
        status: LenientDelegationRequestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_delegation_request("code-519", "label-75");
        assert_eq!(item.status, LenientDelegationRequestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_lenient_delegation_request("code-519", "label-75");
        if item.move_to(LenientDelegationRequestStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientDelegationRequestStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_delegation_request("code-519", "label-75");
        item.status = LenientDelegationRequestStatus::Closed;
        assert!(
            item.move_to(LenientDelegationRequestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
