use super::*;

use acme_kit::text;

/// Where a dns record summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DnsRecordSummaryStatus {
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

impl std::fmt::Display for DnsRecordSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSummary {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub approved: bool,
    pub window_days: i64,
    pub revision: i64,
    pub notes: String,
    pub category: String,
    pub starts_at: i64,
    pub duration_minutes: i64,
    pub status: DnsRecordSummaryStatus,
}

impl DnsRecordSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordSummaryStatus) -> bool {
        match self.status {
            DnsRecordSummaryStatus::Draft => {
                return next == DnsRecordSummaryStatus::Active || next == DnsRecordSummaryStatus::Suspended;
            }
            DnsRecordSummaryStatus::Pending => {
                match next {
                    DnsRecordSummaryStatus::Active | DnsRecordSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordSummaryStatus::Active => {
                return next == DnsRecordSummaryStatus::Suspended || next == DnsRecordSummaryStatus::Closed;
            }
            DnsRecordSummaryStatus::Suspended => {
                match next {
                    DnsRecordSummaryStatus::Closed => {
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

    pub fn move_to(&mut self, next: DnsRecordSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_load(&self) -> f64 {
        ((self.starts_at as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordSummary {
        DnsRecordSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            tags: self.tags.clone(),
            approved: self.approved,
            window_days: self.window_days,
            revision: self.revision,
            notes: self.notes.clone(),
            category: self.category.clone(),
            starts_at: self.starts_at,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a dns record summary in status draft.
pub fn new_dns_record_summary(code: &str, title: &str) -> DnsRecordSummary {
    DnsRecordSummary {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        tags: Vec::new(),
        approved: false,
        window_days: 0,
        revision: 0,
        notes: "".to_string(),
        category: "".to_string(),
        starts_at: 0,
        duration_minutes: 0,
        status: DnsRecordSummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_dns_record_summary("code-170", "title-644");
        assert_eq!(item.status, DnsRecordSummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_dns_record_summary("code-170", "title-644");
        if item.move_to(DnsRecordSummaryStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordSummaryStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_dns_record_summary("code-170", "title-644");
        item.status = DnsRecordSummaryStatus::Closed;
        assert!(
            item.move_to(DnsRecordSummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
