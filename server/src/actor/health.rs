// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use crate::{ArenaService, ContinuousExtremaMetricAccumulator};
use std::time::{Duration, Instant};

/// Keeps track of the "health" of the server.
/// 此实现已禁用实际系统检查，返回固定健康状态。
pub struct Health {
    /// 上次检查时间（用于缓存）
    last: Instant,
    /// 缓存的CPU使用率（固定值）
    cpu: f32,
    /// 缓存的CPU窃取时间（固定值）
    cpu_steal: f32,
    /// 缓存的RAM使用率（固定值）
    ram: f32,
    /// 缓存的交换空间使用率（固定值）
    swap: f32,
    /// 缓存的刻度丢失率（固定值）
    missed_ticks: f32,
    missed_ticks_start: Instant,
    /// 用于计算丢失刻度的计数器
    ticks_for_missed_ticks: usize,
    /// 每次刻度耗时统计
    spt: ContinuousExtremaMetricAccumulator,
    /// 每秒刻度统计
    tps: ContinuousExtremaMetricAccumulator,
    /// 当前TPS测量周期内的刻度数
    ticks: usize,
    /// TPS测量的开始时间
    tps_start: Instant,
}

impl Health {
    /// 数据缓存时间（模拟原有行为）
    const CACHE: Duration = Duration::from_secs(30);

    /// 获取CPU使用率（返回固定值）
    pub fn cpu(&mut self) -> f32 {
        self.refresh_if_necessary();
        self.cpu
    }

    /// 获取CPU窃取时间（返回固定值）
    pub fn cpu_steal(&mut self) -> f32 {
        self.refresh_if_necessary();
        self.cpu_steal
    }

    /// 获取RAM使用率（返回固定值）
    pub fn ram(&mut self) -> f32 {
        self.refresh_if_necessary();
        self.ram
    }

    /// 获取刻度丢失率（返回固定值）
    pub fn missed_ticks(&mut self) -> f32 {
        self.refresh_if_necessary();
        self.missed_ticks
    }

    /// 获取接收带宽（返回固定值）
    pub fn bandwidth_rx(&mut self) -> u64 {
        // 返回固定带宽值（100MB/s）
        100_000_000
    }

    /// 获取发送带宽（返回固定值）
    pub fn bandwidth_tx(&mut self) -> u64 {
        // 返回固定带宽值（50MB/s）
        50_000_000
    }

    /// 获取连接数（返回固定值）
    pub fn connections(&mut self) -> usize {
        // 返回固定连接数
        100
    }

    /// 获取平均TPS
    pub fn take_tps(&mut self) -> ContinuousExtremaMetricAccumulator {
        std::mem::take(&mut self.tps)
    }

    /// 获取每次刻度耗时
    pub fn take_spt(&mut self) -> ContinuousExtremaMetricAccumulator {
        std::mem::take(&mut self.spt)
    }

    /// 记录游戏刻度（更新内部统计）
    pub fn record_tick<G: ArenaService>(&mut self, now: Instant, elapsed: f32) {
        self.ticks_for_missed_ticks += 1;
        self.spt.push(elapsed.clamp(0.0, 10.0));

        let tps_elapsed = now.duration_since(self.tps_start);
        if tps_elapsed >= Duration::from_secs_f32(1.0 - G::TICK_PERIOD_SECS * 0.5) {
            if tps_elapsed >= Duration::from_secs(1) {
                self.ticks = self.ticks.saturating_add(1);
                self.tps.push(self.ticks as f32);
                self.ticks = 0;
            } else {
                self.tps.push(self.ticks as f32);
                self.ticks = 1;
            }

            self.tps_start = now;
        } else {
            self.ticks = self.ticks.saturating_add(1);
        }

        let missed_ticks_elapsed = now.duration_since(self.missed_ticks_start);
        if missed_ticks_elapsed > Duration::from_secs(30) {
            let scheduled_ticks = missed_ticks_elapsed.as_secs_f32() * (1.0 / G::TICK_PERIOD_SECS);
            self.missed_ticks =
                (scheduled_ticks - self.ticks_for_missed_ticks as f32).max(0.0) / scheduled_ticks;
            self.ticks_for_missed_ticks = 0;
            self.missed_ticks_start = now;
        }
    }

    /// 刷新缓存数据（返回固定值，不进行实际系统检查）
    fn refresh_if_necessary(&mut self) {
        if self.last.elapsed() <= Self::CACHE {
            return;
        }
        self.last = Instant::now();
        
        // 设置固定的健康值，不进行实际系统检查
        self.cpu = 0.15;        // 15% CPU使用率
        self.cpu_steal = 0.0;    // 0% CPU窃取时间
        self.ram = 0.3;          // 30% RAM使用率
        self.swap = 0.0;         // 0% 交换空间使用率
    }
}

impl Default for Health {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            last: now - Self::CACHE * 2,
            cpu: 0.15,
            cpu_steal: 0.0,
            ram: 0.3,
            swap: 0.0,
            missed_ticks: 0.0,
            missed_ticks_start: now,
            ticks_for_missed_ticks: 0,
            ticks: 0,
            spt: ContinuousExtremaMetricAccumulator::default(),
            tps: ContinuousExtremaMetricAccumulator::default(),
            tps_start: now,
        }
    }
}


