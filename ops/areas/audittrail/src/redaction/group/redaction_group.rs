use super::*;

use acme_kit::text;

/// Where a redaction group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RedactionGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for RedactionGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction group as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionGroup {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub name: String,
    pub channel: String,
    pub region: String,
    pub fingerprint: String,
    pub created_at: i64,
    pub quantity: i64,
    pub expires_at: i64,
    pub status: RedactionGroupStatus,
}

impl RedactionGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionGroupStatus) -> bool {
        match self.status {
            RedactionGroupStatus::Idle => {
                return next == RedactionGroupStatus::Warming || next == RedactionGroupStatus::Stopped;
            }
            RedactionGroupStatus::Warming => {
                return next == RedactionGroupStatus::Ready;
            }
            RedactionGroupStatus::Ready => {
                return next == RedactionGroupStatus::Draining || next == RedactionGroupStatus::Stopped;
            }
            RedactionGroupStatus::Draining => {
                return next == RedactionGroupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.batch_size as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionGroup {
        RedactionGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            name: self.name.clone(),
            channel: self.channel.clone(),
            region: self.region.clone(),
            fingerprint: self.fingerprint.clone(),
            created_at: self.created_at,
            quantity: self.quantity,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a redaction group in status idle.
pub fn new_redaction_group(code: &str, label: &str) -> RedactionGroup {
    RedactionGroup {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        name: "".to_string(),
        channel: "".to_string(),
        region: "".to_string(),
        fingerprint: "".to_string(),
        created_at: 0,
        quantity: 0,
        expires_at: 0,
        status: RedactionGroupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_redaction_group("code-229", "label-942");
        assert_eq!(item.status, RedactionGroupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_redaction_group("code-229", "label-942");
        if item.move_to(RedactionGroupStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionGroupStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_redaction_group("code-229", "label-942");
        item.status = RedactionGroupStatus::Stopped;
        assert!(
            item.move_to(RedactionGroupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
