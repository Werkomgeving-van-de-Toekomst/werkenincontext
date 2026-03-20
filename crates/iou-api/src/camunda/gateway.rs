//! Zeebe gRPC client wrapper (geen OAuth; lokaal docker-compose).

use std::time::Duration;
use zeebe_rs::{Client, ClientBuilderError};

/// Gateway naar Zeebe voor processtart en correlatieberichten.
#[derive(Clone)]
pub struct CamundaGateway {
    client: Client,
    pub bpmn_process_id: String,
}

impl CamundaGateway {
    /// Verwacht `ZEEBE_ADDRESS` als `host:poort` (standaard `127.0.0.1:26500`).
    pub async fn from_env() -> Result<Self, String> {
        let addr = std::env::var("ZEEBE_ADDRESS").unwrap_or_else(|_| "127.0.0.1:26500".to_string());
        let (host, port) = parse_zeebe_address(&addr)?;
        let bpmn_process_id = std::env::var("CAMUNDA_BPMN_PROCESS_ID")
            .unwrap_or_else(|_| "DocumentPipeline".to_string());

        let http_base = format!("http://{}", host);
        let client = Client::builder()
            .with_address(&http_base, port)
            .build()
            .await
            .map_err(|e: ClientBuilderError| e.to_string())?;

        Ok(Self {
            client,
            bpmn_process_id,
        })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Start het documentpipeline-proces met procesvariabelen (align WorkflowContext / externe callbacks).
    pub async fn start_document_pipeline(
        &self,
        document_id: uuid::Uuid,
        domain_id: &str,
        document_type: &str,
        template_id: &str,
        initiator_user_id: uuid::Uuid,
        workflow_version: &str,
        run_deep_agent: bool,
    ) -> Result<(i64, i64), String> {
        let doc_str = document_id.to_string();
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct StartVars {
            document_id: String,
            /// Zelfde als domain_id; expliciet voor tenant-scoped integraties.
            tenant: String,
            domain_id: String,
            document_type: String,
            template_id: String,
            initiator_user_id: String,
            workflow_version: String,
            /// Correlatie voor callbacks (gelijk aan documentId).
            correlation_key: String,
            run_deep_agent: bool,
        }

        let vars = StartVars {
            document_id: doc_str.clone(),
            tenant: domain_id.to_string(),
            domain_id: domain_id.to_string(),
            document_type: document_type.to_string(),
            template_id: template_id.to_string(),
            initiator_user_id: initiator_user_id.to_string(),
            workflow_version: workflow_version.to_string(),
            correlation_key: doc_str,
            run_deep_agent,
        };

        let res = self
            .client
            .create_process_instance()
            .with_bpmn_process_id(self.bpmn_process_id.clone())
            .with_variables(vars)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Ok((res.process_instance_key(), res.process_definition_key()))
    }

    /// Publiceert `document_approved` zodat het intermediate catch event verder gaat.
    pub async fn publish_document_approved(
        &self,
        document_id: uuid::Uuid,
        approved: bool,
    ) -> Result<(), String> {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct MsgVars {
            approved: bool,
        }

        self.client
            .publish_message()
            .with_name("document_approved".to_string())
            .with_correlation_key(document_id.to_string())
            .with_time_to_live(Duration::from_secs(24 * 3600))
            .with_variables(MsgVars { approved })
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

fn parse_zeebe_address(addr: &str) -> Result<(String, u16), String> {
    let addr = addr.trim();
    if let Some((h, p)) = addr.rsplit_once(':') {
        let port: u16 = p.parse().map_err(|_| format!("ongeldige ZEEBE_ADDRESS poort: {addr}"))?;
        return Ok((h.to_string(), port));
    }
    Err(format!("ZEEBE_ADDRESS verwacht host:poort, kreeg: {addr}"))
}
