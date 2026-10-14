use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a isolation record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationRecordStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl IsolationRecordStatus {
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

impl std::fmt::Display for IsolationRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRecord {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub ttl_seconds: i64,
    pub attempts: i64,
    pub amount: f64,
    pub contact_email: String,
    pub fingerprint: String,
    pub batch_size: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: IsolationRecordStatus,
}

impl IsolationRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationRecordStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationRecordStatus) -> bool {
        match self.status {
            IsolationRecordStatus::Idle => {
                match next {
                    IsolationRecordStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationRecordStatus::Warming => {
                match next {
                    IsolationRecordStatus::Ready | IsolationRecordStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationRecordStatus::Ready => {
                match next {
                    IsolationRecordStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationRecordStatus::Draining => {
                return next == IsolationRecordStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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

    pub fn adjusted_value(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationRecord {
        IsolationRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            ttl_seconds: self.ttl_seconds,
            attempts: self.attempts,
            amount: self.amount,
            contact_email: self.contact_email.clone(),
            fingerprint: self.fingerprint.clone(),
            batch_size: self.batch_size,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a isolation record in status idle.
pub fn new_isolation_record(code: &str, title: &str) -> IsolationRecord {
    IsolationRecord {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        ttl_seconds: 0,
        attempts: 0,
        amount: 0.0,
        contact_email: "".to_string(),
        fingerprint: "".to_string(),
        batch_size: 0,
        attributes: BTreeMap::new(),
        status: IsolationRecordStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_isolation_record("code-945", "title-391");
        assert_eq!(item.status, IsolationRecordStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_isolation_record("code-945", "title-391");
        if item.move_to(IsolationRecordStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationRecordStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_isolation_record("code-945", "title-391");
        item.status = IsolationRecordStatus::Stopped;
        assert!(
            item.move_to(IsolationRecordStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
