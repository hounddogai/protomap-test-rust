use super::*;

use acme_kit::text;

/// Where a silence batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SilenceBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SilenceBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One silence batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceBatch {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub category: String,
    pub limit: i64,
    pub source: String,
    pub status: SilenceBatchStatus,
}

impl SilenceBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceBatchStatus) -> bool {
        match self.status {
            SilenceBatchStatus::Requested => {
                return next == SilenceBatchStatus::Granted || next == SilenceBatchStatus::Expired;
            }
            SilenceBatchStatus::Granted => {
                match next {
                    SilenceBatchStatus::Revoked | SilenceBatchStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceBatchStatus::Revoked => {
                return next == SilenceBatchStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.limit as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceBatch {
        SilenceBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            category: self.category.clone(),
            limit: self.limit,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a silence batch in status requested.
pub fn new_silence_batch(code: &str, label: &str) -> SilenceBatch {
    SilenceBatch {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        category: "".to_string(),
        limit: 0,
        source: "".to_string(),
        status: SilenceBatchStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_silence_batch("code-808", "label-773");
        assert_eq!(item.status, SilenceBatchStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_silence_batch("code-808", "label-773");
        if item.move_to(SilenceBatchStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceBatchStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_silence_batch("code-808", "label-773");
        item.status = SilenceBatchStatus::Expired;
        assert!(
            item.move_to(SilenceBatchStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
