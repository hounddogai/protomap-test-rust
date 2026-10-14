use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a flow log rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogRollupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FlowLogRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FlowLogRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogRollup {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub retries: i64,
    pub starts_at: i64,
    pub version: i64,
    pub host: String,
    pub approved: bool,
    pub attributes: BTreeMap<String, String>,
    pub description: String,
    pub status: FlowLogRollupStatus,
}

impl FlowLogRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogRollupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogRollupStatus) -> bool {
        match self.status {
            FlowLogRollupStatus::Requested => {
                return next == FlowLogRollupStatus::Expired;
            }
            FlowLogRollupStatus::Granted => {
                match next {
                    FlowLogRollupStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogRollupStatus::Revoked => {
                return next == FlowLogRollupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FlowLogRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "flow log rollup", self.code)
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
        ((self.starts_at as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogRollup {
        FlowLogRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            retries: self.retries,
            starts_at: self.starts_at,
            version: self.version,
            host: self.host.clone(),
            approved: self.approved,
            attributes: self.attributes.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a flow log rollup in status requested.
pub fn new_flow_log_rollup(code: &str, title: &str) -> FlowLogRollup {
    FlowLogRollup {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        retries: 0,
        starts_at: 0,
        version: 0,
        host: "".to_string(),
        approved: false,
        attributes: BTreeMap::new(),
        description: "".to_string(),
        status: FlowLogRollupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_flow_log_rollup("code-671", "title-48");
        assert_eq!(item.status, FlowLogRollupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_flow_log_rollup("code-671", "title-48");
        if item.move_to(FlowLogRollupStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogRollupStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_flow_log_rollup("code-671", "title-48");
        item.status = FlowLogRollupStatus::Expired;
        assert!(
            item.move_to(FlowLogRollupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
