use super::*;

use acme_kit::text;

/// Where a dns record baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordBaselineStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DnsRecordBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DnsRecordBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dns record baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBaseline {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub threshold: f64,
    pub issuer: String,
    pub label: String,
    pub status: DnsRecordBaselineStatus,
}

impl DnsRecordBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordBaselineStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordBaselineStatus) -> bool {
        match self.status {
            DnsRecordBaselineStatus::Proposed => {
                return next == DnsRecordBaselineStatus::Approved;
            }
            DnsRecordBaselineStatus::Approved => {
                match next {
                    DnsRecordBaselineStatus::Rejected | DnsRecordBaselineStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordBaselineStatus::Rejected => {
                return next == DnsRecordBaselineStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.threshold + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordBaseline {
        DnsRecordBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            threshold: self.threshold,
            issuer: self.issuer.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a dns record baseline in status proposed.
pub fn new_dns_record_baseline(code: &str, title: &str) -> DnsRecordBaseline {
    DnsRecordBaseline {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        threshold: 0.0,
        issuer: "".to_string(),
        label: "".to_string(),
        status: DnsRecordBaselineStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_dns_record_baseline("code-979", "title-269");
        assert_eq!(item.status, DnsRecordBaselineStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_dns_record_baseline("code-979", "title-269");
        if item.move_to(DnsRecordBaselineStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordBaselineStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_dns_record_baseline("code-979", "title-269");
        item.status = DnsRecordBaselineStatus::Withdrawn;
        assert!(
            item.move_to(DnsRecordBaselineStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
