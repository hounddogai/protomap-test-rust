use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a dns record metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordMetricStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DnsRecordMetricStatus {
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

impl std::fmt::Display for DnsRecordMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dns record metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMetric {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub tags: Vec<String>,
    pub attributes: BTreeMap<String, String>,
    pub window_days: i64,
    pub ttl_seconds: i64,
    pub starts_at: i64,
    pub amount: f64,
    pub ceiling: f64,
    pub status: DnsRecordMetricStatus,
}

impl DnsRecordMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordMetricStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordMetricStatus) -> bool {
        match self.status {
            DnsRecordMetricStatus::New => {
                match next {
                    DnsRecordMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordMetricStatus::Queued => {
                match next {
                    DnsRecordMetricStatus::Done | DnsRecordMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordMetricStatus::Running => {
                match next {
                    DnsRecordMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordMetricStatus::Done => {
                return next == DnsRecordMetricStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn effective_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordMetric {
        DnsRecordMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            tags: self.tags.clone(),
            attributes: self.attributes.clone(),
            window_days: self.window_days,
            ttl_seconds: self.ttl_seconds,
            starts_at: self.starts_at,
            amount: self.amount,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a dns record metric in status new.
pub fn new_dns_record_metric(code: &str, name: &str) -> DnsRecordMetric {
    DnsRecordMetric {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        tags: Vec::new(),
        attributes: BTreeMap::new(),
        window_days: 0,
        ttl_seconds: 0,
        starts_at: 0,
        amount: 0.0,
        ceiling: 0.0,
        status: DnsRecordMetricStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_dns_record_metric("code-103", "name-747");
        assert_eq!(item.status, DnsRecordMetricStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_dns_record_metric("code-103", "name-747");
        if item.move_to(DnsRecordMetricStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordMetricStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_dns_record_metric("code-103", "name-747");
        item.status = DnsRecordMetricStatus::Failed;
        assert!(
            item.move_to(DnsRecordMetricStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
