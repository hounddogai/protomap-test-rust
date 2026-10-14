use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a regional runbook request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRunbookRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalRunbookRequestStatus {
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

impl std::fmt::Display for RegionalRunbookRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional runbook request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookRequest {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub batch_size: i64,
    pub ends_at: i64,
    pub score: f64,
    pub contact_email: String,
    pub status: RegionalRunbookRequestStatus,
}

impl RegionalRunbookRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRunbookRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRunbookRequestStatus) -> bool {
        match self.status {
            RegionalRunbookRequestStatus::Draft => {
                return next == RegionalRunbookRequestStatus::Suspended;
            }
            RegionalRunbookRequestStatus::Pending => {
                return next == RegionalRunbookRequestStatus::Active || next == RegionalRunbookRequestStatus::Suspended;
            }
            RegionalRunbookRequestStatus::Active => {
                match next {
                    RegionalRunbookRequestStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRunbookRequestStatus::Suspended => {
                return next == RegionalRunbookRequestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalRunbookRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional runbook request {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.ends_at as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRunbookRequest {
        RegionalRunbookRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            batch_size: self.batch_size,
            ends_at: self.ends_at,
            score: self.score,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional runbook request in status draft.
pub fn new_regional_runbook_request(code: &str, label: &str) -> RegionalRunbookRequest {
    RegionalRunbookRequest {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        batch_size: 0,
        ends_at: 0,
        score: 0.0,
        contact_email: "".to_string(),
        status: RegionalRunbookRequestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_runbook_request("code-313", "label-434");
        assert_eq!(item.status, RegionalRunbookRequestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_regional_runbook_request("code-313", "label-434");
        if item.move_to(RegionalRunbookRequestStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRunbookRequestStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_runbook_request("code-313", "label-434");
        item.status = RegionalRunbookRequestStatus::Closed;
        assert!(
            item.move_to(RegionalRunbookRequestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
