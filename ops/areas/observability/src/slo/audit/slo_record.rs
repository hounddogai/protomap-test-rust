use super::*;

use acme_kit::text;

/// Where a slo record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloRecordStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SloRecordStatus {
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

impl std::fmt::Display for SloRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a slo record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SloRecord {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub amount: f64,
    pub checksum: String,
    pub phone: String,
    pub status: SloRecordStatus,
}

impl SloRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloRecordStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloRecordStatus) -> bool {
        match self.status {
            SloRecordStatus::Idle => {
                match next {
                    SloRecordStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloRecordStatus::Warming => {
                return next == SloRecordStatus::Draining;
            }
            SloRecordStatus::Ready => {
                match next {
                    SloRecordStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloRecordStatus::Draining => {
                return next == SloRecordStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.grace_days as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloRecord {
        SloRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            amount: self.amount,
            checksum: self.checksum.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a slo record in status idle.
pub fn new_slo_record(code: &str, title: &str) -> SloRecord {
    SloRecord {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        amount: 0.0,
        checksum: "".to_string(),
        phone: "".to_string(),
        status: SloRecordStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_slo_record("code-714", "title-212");
        assert_eq!(item.status, SloRecordStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_slo_record("code-714", "title-212");
        if item.move_to(SloRecordStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloRecordStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_slo_record("code-714", "title-212");
        item.status = SloRecordStatus::Stopped;
        assert!(item.move_to(SloRecordStatus::Idle).is_err(), "expected move to fail");
    }
}
