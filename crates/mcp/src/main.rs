use std::{env, ffi::OsString, process::ExitStatus};

use anyhow::{Context, Result};
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

const DEFAULT_NJU_CLI_BIN: &str = "nju-cli";
const NJU_CLI_BIN_ENV: &str = "NJU_CLI_BIN";

#[derive(Debug, Clone)]
struct NjuCliMcp {
    nju_cli_bin: OsString,
    tool_router: ToolRouter<Self>,
}

impl NjuCliMcp {
    fn from_env() -> Self {
        Self::new(default_nju_cli_bin())
    }

    fn new(nju_cli_bin: OsString) -> Self {
        Self {
            nju_cli_bin,
            tool_router: Self::tool_router(),
        }
    }
}

fn default_nju_cli_bin() -> OsString {
    env::var_os(NJU_CLI_BIN_ENV)
        .or_else(|| option_env!("NJU_CLI_BIN").map(OsString::from))
        .unwrap_or_else(|| DEFAULT_NJU_CLI_BIN.into())
}

#[derive(Debug, Deserialize, JsonSchema)]
struct NjuCliRequest {
    /// 传给 nju-cli 的参数，不包含二进制名本身。
    args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
struct NjuCliResponse {
    /// nju-cli 是否以 0 退出。
    success: bool,
    /// 进程退出码；如果进程被信号终止则为空。
    code: Option<i32>,
    /// nju-cli 的标准输出。
    stdout: String,
    /// nju-cli 的标准错误。
    stderr: String,
}

#[tool_router]
impl NjuCliMcp {
    #[tool(
        name = "nju-cli",
        description = "Run nju-cli with the provided argument list. Example: {\"args\":[\"academic-affairs\",\"calendar\"]}.",
        output_schema = rmcp::handler::server::common::schema_for_output::<NjuCliResponse>()
            .expect("NjuCliResponse must have an object output schema"),
        annotations(
            title = "nju-cli",
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn nju_cli(
        &self,
        Parameters(request): Parameters<NjuCliRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let response = match run_nju_cli(self.nju_cli_bin.clone(), request.args).await {
            Ok(response) => response,
            Err(error) => NjuCliResponse {
                success: false,
                code: None,
                stdout: String::new(),
                stderr: error.to_string(),
            },
        };

        tool_result(response)
    }
}

#[tool_handler]
impl ServerHandler for NjuCliMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: "nju-cli-mcp".to_owned(),
                title: Some("NJU CLI MCP".to_owned()),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                icons: None,
                website_url: Some("https://github.com/nju-cli-org/nju-cli".to_owned()),
            },
            instructions: Some(
                "Use the `nju-cli` tool with an `args` array containing the command-line arguments after `nju-cli`."
                    .to_owned(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

async fn run_nju_cli(nju_cli_bin: OsString, args: Vec<String>) -> Result<NjuCliResponse> {
    let output = Command::new(&nju_cli_bin)
        .args(args)
        .output()
        .await
        .with_context(|| format!("failed to run {}", nju_cli_bin.to_string_lossy()))?;

    Ok(NjuCliResponse {
        success: output.status.success(),
        code: exit_code(output.status),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn exit_code(status: ExitStatus) -> Option<i32> {
    status.code()
}

fn tool_result(response: NjuCliResponse) -> std::result::Result<CallToolResult, McpError> {
    let structured_content = serde_json::to_value(&response).map_err(|error| {
        McpError::internal_error(
            format!("failed to serialize nju-cli response: {error}"),
            None,
        )
    })?;

    Ok(CallToolResult {
        content: vec![Content::text(render_response(&response))],
        structured_content: Some(structured_content),
        is_error: Some(!response.success),
        meta: None,
    })
}

fn render_response(response: &NjuCliResponse) -> String {
    let mut rendered = format!(
        "success: {}\ncode: {}\n",
        response.success,
        response
            .code
            .map(|code| code.to_string())
            .unwrap_or_else(|| "null".to_owned())
    );

    if !response.stdout.is_empty() {
        rendered.push_str("\nstdout:\n");
        rendered.push_str(&response.stdout);
        if !response.stdout.ends_with('\n') {
            rendered.push('\n');
        }
    }

    if !response.stderr.is_empty() {
        rendered.push_str("\nstderr:\n");
        rendered.push_str(&response.stderr);
        if !response.stderr.ends_with('\n') {
            rendered.push('\n');
        }
    }

    rendered
}

#[tokio::main]
async fn main() -> Result<()> {
    let service = NjuCliMcp::from_env()
        .serve(stdio())
        .await
        .context("failed to start nju-cli MCP server")?;
    service
        .waiting()
        .await
        .context("nju-cli MCP server failed")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_response_includes_status_and_streams() {
        let rendered = render_response(&NjuCliResponse {
            success: false,
            code: Some(2),
            stdout: "out".to_owned(),
            stderr: "err".to_owned(),
        });

        assert!(rendered.contains("success: false"));
        assert!(rendered.contains("code: 2"));
        assert!(rendered.contains("stdout:\nout\n"));
        assert!(rendered.contains("stderr:\nerr\n"));
    }
}
