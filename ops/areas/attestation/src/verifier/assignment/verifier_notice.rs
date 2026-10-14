use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a verifier notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierNoticeStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl VerifierNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for VerifierNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verifier notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierNotice {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub locale: String,
    pub contact_email: String,
    pub host: String,
    pub ratio: f64,
    pub limit: i64,
    pub country: String,
    pub burst: i64,
    pub ttl_seconds: i64,
    pub status: VerifierNoticeStatus,
}

impl VerifierNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierNoticeStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierNoticeStatus) -> bool {
        match self.status {
            VerifierNoticeStatus::Planned => {
                return next == VerifierNoticeStatus::Scheduled || next == VerifierNoticeStatus::Completed;
            }
            VerifierNoticeStatus::Scheduled => {
                match next {
                    VerifierNoticeStatus::InProgress | VerifierNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierNoticeStatus::InProgress => {
                return next == VerifierNoticeStatus::Cancelled;
            }
            VerifierNoticeStatus::Completed => {
                match next {
                    VerifierNoticeStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: VerifierNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
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

    pub fn projected_volume(&self) -> f64 {
        ((self.ratio - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierNotice {
        VerifierNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            locale: self.locale.clone(),
            contact_email: self.contact_email.clone(),
            host: self.host.clone(),
            ratio: self.ratio,
            limit: self.limit,
            country: self.country.clone(),
            burst: self.burst,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a verifier notice in status planned.
pub fn new_verifier_notice(code: &str, name: &str) -> VerifierNotice {
    VerifierNotice {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        locale: "".to_string(),
        contact_email: "".to_string(),
        host: "".to_string(),
        ratio: 0.0,
        limit: 0,
        country: "".to_string(),
        burst: 0,
        ttl_seconds: 0,
        status: VerifierNoticeStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_verifier_notice("code-244", "name-165");
        assert_eq!(item.status, VerifierNoticeStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_verifier_notice("code-244", "name-165");
        if item.move_to(VerifierNoticeStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierNoticeStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_verifier_notice("code-244", "name-165");
        item.status = VerifierNoticeStatus::Cancelled;
        assert!(
            item.move_to(VerifierNoticeStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
