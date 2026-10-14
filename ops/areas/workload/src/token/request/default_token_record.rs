use super::*;

use acme_kit::text;

/// Where a default token record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultTokenRecordStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultTokenRecordStatus {
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

impl std::fmt::Display for DefaultTokenRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default token record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTokenRecord {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub external_ref: String,
    pub quantity: i64,
    pub rotation_days: i64,
    pub burst: i64,
    pub status: DefaultTokenRecordStatus,
}

impl DefaultTokenRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultTokenRecordStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultTokenRecordStatus) -> bool {
        match self.status {
            DefaultTokenRecordStatus::Idle => {
                return next == DefaultTokenRecordStatus::Warming || next == DefaultTokenRecordStatus::Stopped;
            }
            DefaultTokenRecordStatus::Warming => {
                match next {
                    DefaultTokenRecordStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTokenRecordStatus::Ready => {
                return next == DefaultTokenRecordStatus::Stopped;
            }
            DefaultTokenRecordStatus::Draining => {
                return next == DefaultTokenRecordStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultTokenRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default token record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default token record", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.batch_size as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultTokenRecord {
        DefaultTokenRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            external_ref: self.external_ref.clone(),
            quantity: self.quantity,
            rotation_days: self.rotation_days,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default token record in status idle.
pub fn new_default_token_record(code: &str, title: &str) -> DefaultTokenRecord {
    DefaultTokenRecord {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        external_ref: "".to_string(),
        quantity: 0,
        rotation_days: 0,
        burst: 0,
        status: DefaultTokenRecordStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_token_record("code-646", "title-516");
        assert_eq!(item.status, DefaultTokenRecordStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_default_token_record("code-646", "title-516");
        if item.move_to(DefaultTokenRecordStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultTokenRecordStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_token_record("code-646", "title-516");
        item.status = DefaultTokenRecordStatus::Stopped;
        assert!(
            item.move_to(DefaultTokenRecordStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
