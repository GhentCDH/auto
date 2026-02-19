/*!
* A Kuma Uptime connection using crate `kuma-client`.
*
* This allows the backend to sync healthchecks with the Kuma instance automatically.
*
* The database of Auto becomes the single source of truth.
*/

use futures::{StreamExt as _, TryStreamExt as _, stream};
use kuma_client::{
    Client,
    monitor::{HttpAuth, HttpBodyEncoding, HttpMethod, MonitorHttp},
};

use crate::{AppState, Result, models::HealthcheckWithRelations, service};

pub async fn sync_healthchecks_to_kuma(state: AppState) -> Result<()> {
    let client = Client::connect(kuma_client::Config {
        url: state.config.kuma_url.clone(),
        username: Some(state.config.kuma_username.clone()),
        password: Some(state.config.kuma_password.clone()),
        ..Default::default()
    })
    .await?;

    let healthchecks = service::healthcheck::get_all_with_relations(&state.pool).await?;

    // sync the healthchecks concurrently (10 outgoing requests at a time)
    stream::iter(healthchecks)
        .map(Ok)
        .try_for_each_concurrent(10, |hc| sync_healthcheck_to_kuma(&client, hc))
        .await
}

async fn sync_healthcheck_to_kuma(
    client: &Client,
    healthcheck_with_relations: HealthcheckWithRelations,
) -> Result<()> {
    let url = healthcheck_with_relations.url();
    let healthcheck = healthcheck_with_relations.healthcheck;
    let method = match healthcheck.method.as_str() {
        "HEAD" => HttpMethod::HEAD,
        "POST" => HttpMethod::POST,
        "DELETE" => HttpMethod::DELETE,
        "PUT" => HttpMethod::PUT,
        "OPTIONS" => HttpMethod::OPTIONS,
        "PATCH" => HttpMethod::PATCH,
        _ => HttpMethod::GET,
    };
    let body_encoding = match healthcheck.request_body_encoding.as_str() {
        "x-www-form-urlencoded" => HttpBodyEncoding::Form,
        "XML" => HttpBodyEncoding::Xml,
        _ => HttpBodyEncoding::Json,
    };

    let auth = match (healthcheck.http_auth_user, healthcheck.http_auth_pass) {
        (Some(user), Some(pass)) => HttpAuth::Basic {
            username: Some(user),
            password: Some(pass),
        },
        _ => HttpAuth::None,
    };

    if let Some(kuma_id) = healthcheck.kuma_id {
        client
            .edit_monitor(MonitorHttp {
                id: Some(kuma_id),
                name: Some(healthcheck.name),
                description: healthcheck.notes,
                interval: Some(healthcheck.interval),
                active: Some(healthcheck.is_enabled),
                max_retries: Some(healthcheck.retry),
                retry_interval: Some(healthcheck.retry_interval),
                url: Some(url),
                timeout: Some(healthcheck.timeout_seconds),
                method: Some(method),
                http_body_encoding: Some(body_encoding),
                body: healthcheck.request_body,
                headers: healthcheck.headers,
                auth: Some(auth),
                ..Default::default()
            })
            .await?;
        Ok(())
    } else {
        client
            .add_monitor(MonitorHttp {
                name: Some(healthcheck.name),
                description: healthcheck.notes,
                interval: Some(healthcheck.interval),
                active: Some(healthcheck.is_enabled),
                max_retries: Some(healthcheck.retry),
                retry_interval: Some(healthcheck.retry_interval),
                url: Some(url),
                timeout: Some(healthcheck.timeout_seconds),
                method: Some(method),
                http_body_encoding: Some(body_encoding),
                body: healthcheck.request_body,
                headers: healthcheck.headers,
                auth: Some(auth),
                ..Default::default()
            })
            .await?;
        Ok(())
    }
}
