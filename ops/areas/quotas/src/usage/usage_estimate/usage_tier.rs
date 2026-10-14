use super::*;

use acme_kit::text;

/// Where a usage tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageTierStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl UsageTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for UsageTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage tier as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageTier {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub issuer: String,
    pub floor_value: f64,
    pub ratio: f64,
    pub threshold: f64,
    pub contact_email: String,
    pub discount: f64,
    pub status: UsageTierStatus,
}

impl UsageTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageTierStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageTierStatus) -> bool {
        match self.status {
            UsageTierStatus::Requested => {
                return next == UsageTierStatus::Granted || next == UsageTierStatus::Revoked;
            }
            UsageTierStatus::Granted => {
                match next {
                    UsageTierStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageTierStatus::Revoked => {
                match next {
                    UsageTierStatus::Expired => {
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

    pub fn move_to(&mut self, next: UsageTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.floor_value + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageTier {
        UsageTier {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            issuer: self.issuer.clone(),
            floor_value: self.floor_value,
            ratio: self.ratio,
            threshold: self.threshold,
            contact_email: self.contact_email.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a usage tier in status requested.
pub fn new_usage_tier(code: &str, label: &str) -> UsageTier {
    UsageTier {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        issuer: "".to_string(),
        floor_value: 0.0,
        ratio: 0.0,
        threshold: 0.0,
        contact_email: "".to_string(),
        discount: 0.0,
        status: UsageTierStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_usage_tier("code-821", "label-623");
        assert_eq!(item.status, UsageTierStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_usage_tier("code-821", "label-623");
        if item.move_to(UsageTierStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageTierStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_usage_tier("code-821", "label-623");
        item.status = UsageTierStatus::Expired;
        assert!(
            item.move_to(UsageTierStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
