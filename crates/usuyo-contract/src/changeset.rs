//! 改动提议文件 `.usuyo/changes/<id>.json`（SPEC §1.5.2）。

use serde::{Deserialize, Serialize};

use crate::{ContractError, Extra, Op, OpTarget};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Changeset {
    pub id: String,
    pub created_at: String,
    /// `agent:<name>` 或 `model:<provider 名>`。
    pub proposer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub state: ProposalState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decided_at: Option<String>,
    pub ops: Vec<ChangeOp>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 提议里的一项；`args` 照 [`Op`] 的线上写法。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeOp {
    pub id: String,
    pub op: String,
    pub target: OpTarget,
    /// 提出时目标文件的 `sha256:<hex>`；新建对象为 `null`。
    pub base: Option<String>,
    #[serde(default)]
    pub args: serde_json::Value,
    pub state: DecisionState,
    #[serde(default, skip_serializing_if = "crate::is_false")]
    pub edited: bool,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProposalState {
    Proposed,
    Accepted,
    PartlyAccepted,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DecisionState {
    Proposed,
    Accepted,
    Rejected,
    Conflict,
}

impl ChangeOp {
    /// 解析成带类型的 op。
    pub fn typed(&self) -> Result<Op, ContractError> {
        Op::from_parts(&self.op, &self.args)
    }
}
