use super::*;

use acme_kit::text;

/// How screens and exports show a runbook record.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookRecordView {
    pub code: String,
    pub title_text: String,
    pub archived: String,
    pub burst: String,
    pub margin: String,
    pub revision: String,
    pub ceiling: String,
    pub unit: String,
    pub region: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_runbook_record_view(item: &RunbookRecord) -> RunbookRecordView {
    RunbookRecordView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        archived: format!("{}", item.archived),
        burst: item.burst.to_string(),
        margin: format!("{:.2}", item.margin),
        revision: item.revision.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        unit: item.unit.clone(),
        region: text::truncate(&item.region, 80),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_record_views(items: &[RunbookRecord]) -> Vec<RunbookRecordView> {
    let mut views: Vec<RunbookRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_record_view(item));
    }
    views
}

pub fn sum_runbook_record_view(view: &RunbookRecordView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_burst as f64 + parsed_margin + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_record("code-717", "title-206");
        let view = to_runbook_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
