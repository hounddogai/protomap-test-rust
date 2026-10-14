use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a ttl policy profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyProfileStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TtlPolicyProfileStatus {
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

impl std::fmt::Display for TtlPolicyProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a ttl policy profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyProfile {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub port: i64,
    pub host: String,
    pub maximum: f64,
    pub timezone: String,
    pub attributes: BTreeMap<String, String>,
    pub status: TtlPolicyProfileStatus,
}

impl TtlPolicyProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyProfileStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyProfileStatus) -> bool {
        match self.status {
            TtlPolicyProfileStatus::Draft => {
                match next {
                    TtlPolicyProfileStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyProfileStatus::Pending => {
                return next == TtlPolicyProfileStatus::Suspended;
            }
            TtlPolicyProfileStatus::Active => {
                match next {
                    TtlPolicyProfileStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyProfileStatus::Suspended => {
                return next == TtlPolicyProfileStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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

    pub fn net_load(&self) -> f64 {
        (self.maximum * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyProfile {
        TtlPolicyProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            port: self.port,
            host: self.host.clone(),
            maximum: self.maximum,
            timezone: self.timezone.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a ttl policy profile in status draft.
pub fn new_ttl_policy_profile(code: &str, label: &str) -> TtlPolicyProfile {
    TtlPolicyProfile {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        port: 0,
        host: "".to_string(),
        maximum: 0.0,
        timezone: "".to_string(),
        attributes: BTreeMap::new(),
        status: TtlPolicyProfileStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_ttl_policy_profile("code-968", "label-664");
        assert_eq!(item.status, TtlPolicyProfileStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_ttl_policy_profile("code-968", "label-664");
        if item.move_to(TtlPolicyProfileStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyProfileStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_ttl_policy_profile("code-968", "label-664");
        item.status = TtlPolicyProfileStatus::Closed;
        assert!(
            item.move_to(TtlPolicyProfileStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
