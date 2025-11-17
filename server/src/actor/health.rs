use actix::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

// 健康检查状态
#[derive(Clone, Debug)]
pub struct HealthStatus {
    pub status: String,
    pub last_check: u64,
    pub checks: HashMap<String, CheckResult>,
}

// 检查结果
#[derive(Clone, Debug)]
pub struct CheckResult {
    pub status: String,
    pub message: String,
}

// 健康检查Actor
#[derive(Default)]
pub struct HealthActor;

impl Actor for HealthActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        log::info!("Health actor started (DISABLED)");
    }
}

// 健康检查消息
#[derive(Message)]
#[rtype(result = "HealthStatus")]
pub struct GetHealthStatus;

impl Handler<GetHealthStatus> for HealthActor {
    type Result = HealthStatus;

    fn handle(&mut self, _msg: GetHealthStatus, _ctx: &mut Context<Self>) -> Self::Result {
        // 返回固定的健康状态，不进行任何实际检查
        let mut checks = HashMap::new();
        checks.insert(
            "system".to_string(),
            CheckResult {
                status: "healthy".to_string(),
                message: "Health checks disabled".to_string(),
            },
        );
        checks.insert(
            "network".to_string(),
            CheckResult {
                status: "healthy".to_string(),
                message: "Health checks disabled".to_string(),
            },
        );
        checks.insert(
            "memory".to_string(),
            CheckResult {
                status: "healthy".to_string(),
                message: "Health checks disabled".to_string(),
            },
        );

        HealthStatus {
            status: "healthy".to_string(),
            last_check: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            checks,
        }
    }
}

// 更新健康状态消息
#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct UpdateHealthStatus;

impl Handler<UpdateHealthStatus> for HealthActor {
    type Result = Result<(), String>;

    fn handle(&mut self, _msg: UpdateHealthStatus, _ctx: &mut Context<Self>) -> Self::Result {
        // 不执行任何操作，直接返回成功
        log::debug!("Health status update requested but disabled");
        Ok(())
    }
}

// 健康检查配置
#[derive(Clone, Debug)]
pub struct HealthConfig {
    pub enabled: bool,
    pub check_interval: u64,
    pub storage_path: String,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enabled: false, // 默认禁用
            check_interval: 0,
            storage_path: "./data/health".to_string(),
        }
    }
}


