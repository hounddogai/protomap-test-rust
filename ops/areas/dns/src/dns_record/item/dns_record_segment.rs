use super::*;

use acme_kit::text;

/// Where a dns record segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordSegmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DnsRecordSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DnsRecordSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dns record segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSegment {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub amount: f64,
    pub country: String,
    pub approved: bool,
    pub rotation_days: i64,
    pub limit: i64,
    pub status: DnsRecordSegmentStatus,
}

impl DnsRecordSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordSegmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordSegmentStatus) -> bool {
        match self.status {
            DnsRecordSegmentStatus::Requested => {
                return next == DnsRecordSegmentStatus::Granted || next == DnsRecordSegmentStatus::Expired;
            }
            DnsRecordSegmentStatus::Granted => {
                match next {
                    DnsRecordSegmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordSegmentStatus::Revoked => {
                match next {
                    DnsRecordSegmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: DnsRecordSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.amount + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordSegment {
        DnsRecordSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            amount: self.amount,
            country: self.country.clone(),
            approved: self.approved,
            rotation_days: self.rotation_days,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a dns record segment in status requested.
pub fn new_dns_record_segment(code: &str, title: &str) -> DnsRecordSegment {
    DnsRecordSegment {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        amount: 0.0,
        country: "".to_string(),
        approved: false,
        rotation_days: 0,
        limit: 0,
        status: DnsRecordSegmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_dns_record_segment("code-714", "title-182");
        assert_eq!(item.status, DnsRecordSegmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_dns_record_segment("code-714", "title-182");
        if item.move_to(DnsRecordSegmentStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordSegmentStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_dns_record_segment("code-714", "title-182");
        item.status = DnsRecordSegmentStatus::Expired;
        assert!(
            item.move_to(DnsRecordSegmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
