use super::*;

use acme_kit::text;

/// Where a weighted dns record template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedDnsRecordTemplateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedDnsRecordTemplateStatus {
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

impl std::fmt::Display for WeightedDnsRecordTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted dns record template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDnsRecordTemplate {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub locale: String,
    pub batch_size: i64,
    pub score: f64,
    pub external_ref: String,
    pub status: WeightedDnsRecordTemplateStatus,
}

impl WeightedDnsRecordTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedDnsRecordTemplateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedDnsRecordTemplateStatus) -> bool {
        match self.status {
            WeightedDnsRecordTemplateStatus::New => {
                return next == WeightedDnsRecordTemplateStatus::Running
                    || next == WeightedDnsRecordTemplateStatus::Done;
            }
            WeightedDnsRecordTemplateStatus::Queued => {
                match next {
                    WeightedDnsRecordTemplateStatus::Running | WeightedDnsRecordTemplateStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedDnsRecordTemplateStatus::Running => {
                return next == WeightedDnsRecordTemplateStatus::Done
                    || next == WeightedDnsRecordTemplateStatus::Failed;
            }
            WeightedDnsRecordTemplateStatus::Done => {
                match next {
                    WeightedDnsRecordTemplateStatus::Failed => {
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

    pub fn move_to(&mut self, next: WeightedDnsRecordTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted dns record template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        (self.score * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedDnsRecordTemplate {
        WeightedDnsRecordTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            locale: self.locale.clone(),
            batch_size: self.batch_size,
            score: self.score,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a weighted dns record template in status new.
pub fn new_weighted_dns_record_template(code: &str, title: &str) -> WeightedDnsRecordTemplate {
    WeightedDnsRecordTemplate {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        locale: "".to_string(),
        batch_size: 0,
        score: 0.0,
        external_ref: "".to_string(),
        status: WeightedDnsRecordTemplateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_weighted_dns_record_template("code-597", "title-452");
        assert_eq!(item.status, WeightedDnsRecordTemplateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_weighted_dns_record_template("code-597", "title-452");
        if item.move_to(WeightedDnsRecordTemplateStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedDnsRecordTemplateStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_weighted_dns_record_template("code-597", "title-452");
        item.status = WeightedDnsRecordTemplateStatus::Failed;
        assert!(
            item.move_to(WeightedDnsRecordTemplateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
