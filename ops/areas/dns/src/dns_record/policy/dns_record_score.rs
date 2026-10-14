use super::*;

use acme_kit::text;

/// Where a dns record score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordScoreStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DnsRecordScoreStatus {
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

impl std::fmt::Display for DnsRecordScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dns record score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordScore {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub batch_size: i64,
    pub version: i64,
    pub status: DnsRecordScoreStatus,
}

impl DnsRecordScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordScoreStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordScoreStatus) -> bool {
        match self.status {
            DnsRecordScoreStatus::Idle => {
                match next {
                    DnsRecordScoreStatus::Warming | DnsRecordScoreStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordScoreStatus::Warming => {
                match next {
                    DnsRecordScoreStatus::Ready | DnsRecordScoreStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordScoreStatus::Ready => {
                match next {
                    DnsRecordScoreStatus::Draining | DnsRecordScoreStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordScoreStatus::Draining => {
                match next {
                    DnsRecordScoreStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DnsRecordScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.batch_size as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordScore {
        DnsRecordScore {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            batch_size: self.batch_size,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a dns record score in status idle.
pub fn new_dns_record_score(code: &str, name: &str) -> DnsRecordScore {
    DnsRecordScore {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        batch_size: 0,
        version: 0,
        status: DnsRecordScoreStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_dns_record_score("code-195", "name-403");
        assert_eq!(item.status, DnsRecordScoreStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_dns_record_score("code-195", "name-403");
        if item.move_to(DnsRecordScoreStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordScoreStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_dns_record_score("code-195", "name-403");
        item.status = DnsRecordScoreStatus::Stopped;
        assert!(
            item.move_to(DnsRecordScoreStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
