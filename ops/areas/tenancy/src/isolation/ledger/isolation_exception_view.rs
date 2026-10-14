use super::*;

use acme_kit::text;

/// How screens and exports show a isolation exception.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationExceptionView {
    pub code: String,
    pub name_text: String,
    pub ttl_seconds_text: String,
    pub namespace_text: String,
    pub reviewer: String,
    pub key_id_text: String,
    pub owner_id_text: String,
    pub locale_text: String,
    pub batch_size: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_isolation_exception_view(item: &IsolationException) -> IsolationExceptionView {
    IsolationExceptionView {
        code: text::truncate(&item.code, 120),
        name_text: text::truncate(&item.name, 64),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        namespace_text: item.namespace.clone(),
        reviewer: item.reviewer.clone(),
        key_id_text: item.key_id.clone(),
        owner_id_text: item.owner_id.clone(),
        locale_text: item.locale.clone(),
        batch_size: item.batch_size.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_exception_views(items: &[IsolationException]) -> Vec<IsolationExceptionView> {
    let mut views: Vec<IsolationExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_exception_view(item));
    }
    views
}

pub fn sum_isolation_exception_view(view: &IsolationExceptionView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_batch_size as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_exception("code-566", "name-67");
        let view = to_isolation_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
