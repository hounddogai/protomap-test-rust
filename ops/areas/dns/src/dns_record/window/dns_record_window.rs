use super::*;

use acme_kit::text;

/// Where a dns record window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordWindowStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DnsRecordWindowStatus {
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

impl std::fmt::Display for DnsRecordWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dns record window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordWindow {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub approved: bool,
    pub quota_units: i64,
    pub capacity: i64,
    pub currency: String,
    pub reviewer: String,
    pub active: bool,
    pub source: String,
    pub minimum: f64,
    pub status: DnsRecordWindowStatus,
}

impl DnsRecordWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordWindowStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordWindowStatus) -> bool {
        match self.status {
            DnsRecordWindowStatus::Idle => {
                match next {
                    DnsRecordWindowStatus::Ready | DnsRecordWindowStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordWindowStatus::Warming => {
                match next {
                    DnsRecordWindowStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordWindowStatus::Ready => {
                return next == DnsRecordWindowStatus::Draining;
            }
            DnsRecordWindowStatus::Draining => {
                return next == DnsRecordWindowStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.minimum - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordWindow {
        DnsRecordWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            approved: self.approved,
            quota_units: self.quota_units,
            capacity: self.capacity,
            currency: self.currency.clone(),
            reviewer: self.reviewer.clone(),
            active: self.active,
            source: self.source.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a dns record window in status idle.
pub fn new_dns_record_window(code: &str, title: &str) -> DnsRecordWindow {
    DnsRecordWindow {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        approved: false,
        quota_units: 0,
        capacity: 0,
        currency: "".to_string(),
        reviewer: "".to_string(),
        active: false,
        source: "".to_string(),
        minimum: 0.0,
        status: DnsRecordWindowStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_dns_record_window("code-961", "title-324");
        assert_eq!(item.status, DnsRecordWindowStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_dns_record_window("code-961", "title-324");
        if item.move_to(DnsRecordWindowStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordWindowStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_dns_record_window("code-961", "title-324");
        item.status = DnsRecordWindowStatus::Stopped;
        assert!(
            item.move_to(DnsRecordWindowStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
