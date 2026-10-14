use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled token variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTokenVarianceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledTokenVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledTokenVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled token variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTokenVariance {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub discount: f64,
    pub attributes: BTreeMap<String, String>,
    pub key_id: String,
    pub status: ScheduledTokenVarianceStatus,
}

impl ScheduledTokenVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTokenVarianceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTokenVarianceStatus) -> bool {
        match self.status {
            ScheduledTokenVarianceStatus::Requested => {
                match next {
                    ScheduledTokenVarianceStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTokenVarianceStatus::Granted => {
                match next {
                    ScheduledTokenVarianceStatus::Revoked | ScheduledTokenVarianceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTokenVarianceStatus::Revoked => {
                match next {
                    ScheduledTokenVarianceStatus::Expired => {
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

    pub fn move_to(&mut self, next: ScheduledTokenVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled token variance {} cannot move from {} to {}",
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

    pub fn effective_value(&self) -> f64 {
        ((self.burst as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledTokenVariance {
        ScheduledTokenVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            discount: self.discount,
            attributes: self.attributes.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled token variance in status requested.
pub fn new_scheduled_token_variance(code: &str, label: &str) -> ScheduledTokenVariance {
    ScheduledTokenVariance {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        discount: 0.0,
        attributes: BTreeMap::new(),
        key_id: "".to_string(),
        status: ScheduledTokenVarianceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_token_variance("code-449", "label-664");
        assert_eq!(item.status, ScheduledTokenVarianceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_scheduled_token_variance("code-449", "label-664");
        if item.move_to(ScheduledTokenVarianceStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledTokenVarianceStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_token_variance("code-449", "label-664");
        item.status = ScheduledTokenVarianceStatus::Expired;
        assert!(
            item.move_to(ScheduledTokenVarianceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
