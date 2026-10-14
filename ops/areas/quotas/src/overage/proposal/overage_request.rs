use super::*;

use acme_kit::text;

/// Where a overage request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageRequestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl OverageRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for OverageRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageRequest {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub channel: String,
    pub currency: String,
    pub sequence: i64,
    pub grace_days: i64,
    pub window_days: i64,
    pub limit: i64,
    pub quota_units: i64,
    pub tags: Vec<String>,
    pub status: OverageRequestStatus,
}

impl OverageRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageRequestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageRequestStatus) -> bool {
        match self.status {
            OverageRequestStatus::Proposed => {
                match next {
                    OverageRequestStatus::Approved | OverageRequestStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageRequestStatus::Approved => {
                return next == OverageRequestStatus::Rejected;
            }
            OverageRequestStatus::Rejected => {
                match next {
                    OverageRequestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: OverageRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_total(&self) -> f64 {
        ((self.retries as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageRequest {
        OverageRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            channel: self.channel.clone(),
            currency: self.currency.clone(),
            sequence: self.sequence,
            grace_days: self.grace_days,
            window_days: self.window_days,
            limit: self.limit,
            quota_units: self.quota_units,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a overage request in status proposed.
pub fn new_overage_request(code: &str, label: &str) -> OverageRequest {
    OverageRequest {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        channel: "".to_string(),
        currency: "".to_string(),
        sequence: 0,
        grace_days: 0,
        window_days: 0,
        limit: 0,
        quota_units: 0,
        tags: Vec::new(),
        status: OverageRequestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_overage_request("code-976", "label-938");
        assert_eq!(item.status, OverageRequestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_overage_request("code-976", "label-938");
        if item.move_to(OverageRequestStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageRequestStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_overage_request("code-976", "label-938");
        item.status = OverageRequestStatus::Withdrawn;
        assert!(
            item.move_to(OverageRequestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
