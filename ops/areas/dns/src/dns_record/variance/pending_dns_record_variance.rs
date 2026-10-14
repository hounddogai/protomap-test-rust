use super::*;

use acme_kit::text;

/// Where a pending dns record variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingDnsRecordVarianceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingDnsRecordVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PendingDnsRecordVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending dns record variance as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingDnsRecordVariance {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub discount: f64,
    pub checksum: String,
    pub rotation_days: i64,
    pub source: String,
    pub visible: bool,
    pub status: PendingDnsRecordVarianceStatus,
}

impl PendingDnsRecordVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingDnsRecordVarianceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingDnsRecordVarianceStatus) -> bool {
        match self.status {
            PendingDnsRecordVarianceStatus::Draft => {
                match next {
                    PendingDnsRecordVarianceStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingDnsRecordVarianceStatus::Pending => {
                return next == PendingDnsRecordVarianceStatus::Active;
            }
            PendingDnsRecordVarianceStatus::Active => {
                return next == PendingDnsRecordVarianceStatus::Suspended
                    || next == PendingDnsRecordVarianceStatus::Closed;
            }
            PendingDnsRecordVarianceStatus::Suspended => {
                match next {
                    PendingDnsRecordVarianceStatus::Closed => {
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

    pub fn move_to(&mut self, next: PendingDnsRecordVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending dns record variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending dns record variance", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.discount - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingDnsRecordVariance {
        PendingDnsRecordVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            discount: self.discount,
            checksum: self.checksum.clone(),
            rotation_days: self.rotation_days,
            source: self.source.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending dns record variance in status draft.
pub fn new_pending_dns_record_variance(code: &str, title: &str) -> PendingDnsRecordVariance {
    PendingDnsRecordVariance {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        discount: 0.0,
        checksum: "".to_string(),
        rotation_days: 0,
        source: "".to_string(),
        visible: false,
        status: PendingDnsRecordVarianceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_dns_record_variance("code-907", "title-730");
        assert_eq!(item.status, PendingDnsRecordVarianceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_pending_dns_record_variance("code-907", "title-730");
        if item.move_to(PendingDnsRecordVarianceStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingDnsRecordVarianceStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_dns_record_variance("code-907", "title-730");
        item.status = PendingDnsRecordVarianceStatus::Closed;
        assert!(
            item.move_to(PendingDnsRecordVarianceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
