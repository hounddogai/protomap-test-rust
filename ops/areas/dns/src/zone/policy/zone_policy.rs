use super::*;

use acme_kit::text;

/// Where a zone policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZonePolicyStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ZonePolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ZonePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One zone policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZonePolicy {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub burst: i64,
    pub active: bool,
    pub expires_at: i64,
    pub label: String,
    pub cipher: String,
    pub currency: String,
    pub port: i64,
    pub visible: bool,
    pub status: ZonePolicyStatus,
}

impl ZonePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZonePolicyStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZonePolicyStatus) -> bool {
        match self.status {
            ZonePolicyStatus::Proposed => {
                match next {
                    ZonePolicyStatus::Approved | ZonePolicyStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZonePolicyStatus::Approved => {
                match next {
                    ZonePolicyStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZonePolicyStatus::Rejected => {
                return next == ZonePolicyStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZonePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.revision as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZonePolicy {
        ZonePolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            burst: self.burst,
            active: self.active,
            expires_at: self.expires_at,
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            currency: self.currency.clone(),
            port: self.port,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a zone policy in status proposed.
pub fn new_zone_policy(code: &str, title: &str) -> ZonePolicy {
    ZonePolicy {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        burst: 0,
        active: false,
        expires_at: 0,
        label: "".to_string(),
        cipher: "".to_string(),
        currency: "".to_string(),
        port: 0,
        visible: false,
        status: ZonePolicyStatus::Proposed,
    }
}
