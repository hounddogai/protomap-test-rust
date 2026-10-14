use super::*;

use acme_kit::text;

/// Where a probe template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeTemplateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ProbeTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ProbeTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe template as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeTemplate {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub floor_value: f64,
    pub priority: i64,
    pub key_id: String,
    pub status: ProbeTemplateStatus,
}

impl ProbeTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeTemplateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeTemplateStatus) -> bool {
        match self.status {
            ProbeTemplateStatus::Requested => {
                match next {
                    ProbeTemplateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeTemplateStatus::Granted => {
                return next == ProbeTemplateStatus::Revoked;
            }
            ProbeTemplateStatus::Revoked => {
                match next {
                    ProbeTemplateStatus::Expired => {
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

    pub fn move_to(&mut self, next: ProbeTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "probe template", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.priority as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeTemplate {
        ProbeTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            floor_value: self.floor_value,
            priority: self.priority,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a probe template in status requested.
pub fn new_probe_template(code: &str, label: &str) -> ProbeTemplate {
    ProbeTemplate {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        floor_value: 0.0,
        priority: 0,
        key_id: "".to_string(),
        status: ProbeTemplateStatus::Requested,
    }
}
