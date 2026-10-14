use super::*;

use acme_kit::text;

/// Where a regional dns record segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalDnsRecordSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalDnsRecordSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RegionalDnsRecordSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional dns record segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDnsRecordSegment {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub burst: i64,
    pub updated_at: i64,
    pub capacity: i64,
    pub status: RegionalDnsRecordSegmentStatus,
}

impl RegionalDnsRecordSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalDnsRecordSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalDnsRecordSegmentStatus) -> bool {
        match self.status {
            RegionalDnsRecordSegmentStatus::New => {
                match next {
                    RegionalDnsRecordSegmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDnsRecordSegmentStatus::Queued => {
                match next {
                    RegionalDnsRecordSegmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDnsRecordSegmentStatus::Running => {
                return next == RegionalDnsRecordSegmentStatus::Done || next == RegionalDnsRecordSegmentStatus::Failed;
            }
            RegionalDnsRecordSegmentStatus::Done => {
                match next {
                    RegionalDnsRecordSegmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: RegionalDnsRecordSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional dns record segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.burst as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalDnsRecordSegment {
        RegionalDnsRecordSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            burst: self.burst,
            updated_at: self.updated_at,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional dns record segment in status new.
pub fn new_regional_dns_record_segment(code: &str, label: &str) -> RegionalDnsRecordSegment {
    RegionalDnsRecordSegment {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        burst: 0,
        updated_at: 0,
        capacity: 0,
        status: RegionalDnsRecordSegmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_dns_record_segment("code-323", "label-781");
        assert_eq!(item.status, RegionalDnsRecordSegmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_regional_dns_record_segment("code-323", "label-781");
        if item.move_to(RegionalDnsRecordSegmentStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalDnsRecordSegmentStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_dns_record_segment("code-323", "label-781");
        item.status = RegionalDnsRecordSegmentStatus::Failed;
        assert!(
            item.move_to(RegionalDnsRecordSegmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
