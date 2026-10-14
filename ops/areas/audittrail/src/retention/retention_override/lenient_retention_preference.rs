use super::*;

use acme_kit::text;

/// Where a lenient retention preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRetentionPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientRetentionPreferenceStatus {
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

impl std::fmt::Display for LenientRetentionPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient retention preference as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionPreference {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub batch_size: i64,
    pub key_id: String,
    pub cipher: String,
    pub description: String,
    pub burst: i64,
    pub score: f64,
    pub ends_at: i64,
    pub discount: f64,
    pub status: LenientRetentionPreferenceStatus,
}

impl LenientRetentionPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRetentionPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRetentionPreferenceStatus) -> bool {
        match self.status {
            LenientRetentionPreferenceStatus::Idle => {
                return next == LenientRetentionPreferenceStatus::Warming;
            }
            LenientRetentionPreferenceStatus::Warming => {
                match next {
                    LenientRetentionPreferenceStatus::Draining | LenientRetentionPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRetentionPreferenceStatus::Ready => {
                match next {
                    LenientRetentionPreferenceStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRetentionPreferenceStatus::Draining => {
                match next {
                    LenientRetentionPreferenceStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LenientRetentionPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient retention preference {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.score - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRetentionPreference {
        LenientRetentionPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            batch_size: self.batch_size,
            key_id: self.key_id.clone(),
            cipher: self.cipher.clone(),
            description: self.description.clone(),
            burst: self.burst,
            score: self.score,
            ends_at: self.ends_at,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient retention preference in status idle.
pub fn new_lenient_retention_preference(code: &str, label: &str) -> LenientRetentionPreference {
    LenientRetentionPreference {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        batch_size: 0,
        key_id: "".to_string(),
        cipher: "".to_string(),
        description: "".to_string(),
        burst: 0,
        score: 0.0,
        ends_at: 0,
        discount: 0.0,
        status: LenientRetentionPreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_retention_preference("code-421", "label-78");
        assert_eq!(item.status, LenientRetentionPreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_lenient_retention_preference("code-421", "label-78");
        if item.move_to(LenientRetentionPreferenceStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRetentionPreferenceStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_retention_preference("code-421", "label-78");
        item.status = LenientRetentionPreferenceStatus::Stopped;
        assert!(
            item.move_to(LenientRetentionPreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
