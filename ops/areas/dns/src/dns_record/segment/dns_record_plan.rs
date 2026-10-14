use super::*;

use acme_kit::text;

/// Where a dns record plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordPlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DnsRecordPlanStatus {
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

impl std::fmt::Display for DnsRecordPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record plan as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPlan {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub description: String,
    pub audience: String,
    pub cost: f64,
    pub updated_at: i64,
    pub name: String,
    pub checksum: String,
    pub batch_size: i64,
    pub status: DnsRecordPlanStatus,
}

impl DnsRecordPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordPlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordPlanStatus) -> bool {
        match self.status {
            DnsRecordPlanStatus::New => {
                match next {
                    DnsRecordPlanStatus::Running | DnsRecordPlanStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordPlanStatus::Queued => {
                match next {
                    DnsRecordPlanStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordPlanStatus::Running => {
                match next {
                    DnsRecordPlanStatus::Done | DnsRecordPlanStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordPlanStatus::Done => {
                match next {
                    DnsRecordPlanStatus::Failed => {
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

    pub fn move_to(&mut self, next: DnsRecordPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record plan {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.cost - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordPlan {
        DnsRecordPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            description: self.description.clone(),
            audience: self.audience.clone(),
            cost: self.cost,
            updated_at: self.updated_at,
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a dns record plan in status new.
pub fn new_dns_record_plan(code: &str, title: &str) -> DnsRecordPlan {
    DnsRecordPlan {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        description: "".to_string(),
        audience: "".to_string(),
        cost: 0.0,
        updated_at: 0,
        name: "".to_string(),
        checksum: "".to_string(),
        batch_size: 0,
        status: DnsRecordPlanStatus::New,
    }
}
