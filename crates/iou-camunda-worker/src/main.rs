//! Pollt Zeebe-jobs `iou-run-pipeline` en `iou-deep-agent` en roept `iou-api` aan.
//!
//! Omgeving: `ZEEBE_ADDRESS`, `IOU_API_URL`, `CAMUNDA_WORKER_TOKEN`, optioneel `ZEEBE_DEPLOY_BPMN`.

use std::path::PathBuf;
use std::time::Duration;
use tracing::{error, info};
use zeebe_rs::{ActivatedJob, Client, ClientBuilderError, WorkerError};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let zeebe_addr =
        std::env::var("ZEEBE_ADDRESS").unwrap_or_else(|_| "127.0.0.1:26500".to_string());
    let (host, port) = parse_addr(&zeebe_addr)?;
    let http_base = format!("http://{}", host);

    let api_base = std::env::var("IOU_API_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
    let token = std::env::var("CAMUNDA_WORKER_TOKEN").expect("CAMUNDA_WORKER_TOKEN");

    let client = Client::builder()
        .with_address(&http_base, port)
        .build()
        .await
        .map_err(|e: ClientBuilderError| anyhow::anyhow!(e))?;

    if let Ok(bpmn_path) = std::env::var("ZEEBE_DEPLOY_BPMN") {
        let p = PathBuf::from(&bpmn_path);
        if p.exists() {
            match client
                .deploy_resource()
                .with_resource_file(p)
                .read_resource_files()
                .map_err(|e| anyhow::anyhow!(e))?
                .send()
                .await
            {
                Ok(r) => info!(?r, "BPMN gedeployed van {}", bpmn_path),
                Err(e) => error!("Deploy mislukt: {}", e),
            }
        } else {
            tracing::warn!("ZEEBE_DEPLOY_BPMN pad bestaat niet: {}", bpmn_path);
        }
    }

    let api_run = api_base.clone();
    let tok_run = token.clone();
    let run_pipeline = client
        .worker()
        .with_request_timeout(Duration::from_secs(120))
        .with_job_timeout(Duration::from_secs(600))
        .with_max_jobs_to_activate(2)
        .with_concurrency_limit(2)
        .with_job_type("iou-run-pipeline".to_string())
        .with_handler(move |c, job| {
            let api = api_run.clone();
            let t = tok_run.clone();
            handle_run_pipeline(c, job, api, t)
        })
        .build();

    let api_deep = api_base.clone();
    let tok_deep = token.clone();
    let deep_agent = client
        .worker()
        .with_request_timeout(Duration::from_secs(120))
        .with_job_timeout(Duration::from_secs(900))
        .with_max_jobs_to_activate(2)
        .with_concurrency_limit(1)
        .with_job_type("iou-deep-agent".to_string())
        .with_handler(move |c, job| {
            let api = api_deep.clone();
            let t = tok_deep.clone();
            handle_deep_agent(c, job, api, t)
        })
        .build();

    info!(
        zeebe = %zeebe_addr,
        api = %api_base,
        "iou-camunda-worker gestart"
    );

    let ((), ()) = tokio::join!(run_pipeline.run(), deep_agent.run());
    Ok(())
}

async fn handle_run_pipeline(
    _client: Client,
    job: ActivatedJob,
    api_base: String,
    token: String,
) -> Result<serde_json::Value, WorkerError<()>> {
    let vars: serde_json::Value =
        serde_json::from_str(job.variables()).map_err(|e| WorkerError::FailJob(e.to_string()))?;
    let document_id = vars
        .get("documentId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WorkerError::FailJob("documentId ontbreekt".into()))?;

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| WorkerError::FailJob(e.to_string()))?;

    let url = format!(
        "{}/api/internal/camunda/run-pipeline",
        api_base.trim_end_matches('/')
    );
    let body = serde_json::json!({
        "documentId": document_id,
        "zeebeJobKey": job.key(),
    });

    let resp = http
        .post(&url)
        .header("X-Camunda-Worker-Token", &token)
        .json(&body)
        .send()
        .await
        .map_err(|e| WorkerError::FailJob(e.to_string()))?;

    let status = resp.status();
    if !status.is_success() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(WorkerError::FailJob(format!("API {}: {}", status, txt)));
    }

    resp.json()
        .await
        .map_err(|e| WorkerError::FailJob(e.to_string()))
}

async fn handle_deep_agent(
    _client: Client,
    job: ActivatedJob,
    api_base: String,
    token: String,
) -> Result<serde_json::Value, WorkerError<()>> {
    let vars: serde_json::Value =
        serde_json::from_str(job.variables()).map_err(|e| WorkerError::FailJob(e.to_string()))?;
    let document_id = vars
        .get("documentId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WorkerError::FailJob("documentId ontbreekt".into()))?;

    let hint = vars
        .get("promptHint")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(900))
        .build()
        .map_err(|e| WorkerError::FailJob(e.to_string()))?;

    let url = format!(
        "{}/api/internal/camunda/deep-agent",
        api_base.trim_end_matches('/')
    );
    let mut body = serde_json::json!({
        "documentId": document_id,
        "zeebeJobKey": job.key(),
    });
    if let Some(h) = hint {
        body["promptHint"] = serde_json::Value::String(h);
    }

    let resp = http
        .post(&url)
        .header("X-Camunda-Worker-Token", &token)
        .json(&body)
        .send()
        .await
        .map_err(|e| WorkerError::FailJob(e.to_string()))?;

    let status = resp.status();
    if !status.is_success() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(WorkerError::FailJob(format!("API {}: {}", status, txt)));
    }

    resp.json()
        .await
        .map_err(|e| WorkerError::FailJob(e.to_string()))
}

fn parse_addr(addr: &str) -> anyhow::Result<(String, u16)> {
    let addr = addr.trim();
    let (h, p) = addr
        .rsplit_once(':')
        .ok_or_else(|| anyhow::anyhow!("ZEEBE_ADDRESS verwacht host:poort"))?;
    let port: u16 = p.parse()?;
    Ok((h.to_string(), port))
}
