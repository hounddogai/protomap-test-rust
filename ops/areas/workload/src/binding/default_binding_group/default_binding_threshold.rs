use super::*;

use acme_kit::text;

/// Where a default binding threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultBindingThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultBindingThresholdStatus {
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

impl std::fmt::Display for DefaultBindingThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default binding threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBindingThreshold {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub phone: String,
    pub window_days: i64,
    pub cost: f64,
    pub ratio: f64,
    pub fingerprint: String,
    pub channel: String,
    pub port: i64,
    pub version: i64,
    pub status: DefaultBindingThresholdStatus,
}

impl DefaultBindingThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultBindingThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultBindingThresholdStatus) -> bool {
        match self.status {
            DefaultBindingThresholdStatus::Draft => {
                match next {
                    DefaultBindingThresholdStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultBindingThresholdStatus::Pending => {
                return next == DefaultBindingThresholdStatus::Active
                    || next == DefaultBindingThresholdStatus::Suspended;
            }
            DefaultBindingThresholdStatus::Active => {
                return next == DefaultBindingThresholdStatus::Suspended
                    || next == DefaultBindingThresholdStatus::Closed;
            }
            DefaultBindingThresholdStatus::Suspended => {
                return next == DefaultBindingThresholdStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultBindingThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default binding threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.version as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultBindingThreshold {
        DefaultBindingThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            phone: self.phone.clone(),
            window_days: self.window_days,
            cost: self.cost,
            ratio: self.ratio,
            fingerprint: self.fingerprint.clone(),
            channel: self.channel.clone(),
            port: self.port,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a default binding threshold in status draft.
pub fn new_default_binding_threshold(code: &str, label: &str) -> DefaultBindingThreshold {
    DefaultBindingThreshold {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        phone: "".to_string(),
        window_days: 0,
        cost: 0.0,
        ratio: 0.0,
        fingerprint: "".to_string(),
        channel: "".to_string(),
        port: 0,
        version: 0,
        status: DefaultBindingThresholdStatus::Draft,
    }
}
