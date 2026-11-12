//! `builtin/media`：包着 ffmpeg 和 ffprobe 的转换插件（feature 003 §4.1、§5.3）。
//!
//! - [`plugin`]：转换协议的 `run`、`health`，交给 plugin-kit 的循环。
//! - [`products`]：`probe`、`play`、`audio:pcm16k`；[`keyframes`]：两遍取关键帧。
//! - [`scan`]、[`select`]：第一遍的 stderr 怎么读、选哪几帧（纯函数）。
//! - [`exec`]、[`tools`]、[`probe`]、[`budget`]：起子进程、找程序和拼命令行、读 ffprobe 的 JSON、时间上限。
//! - [`options`]、[`jpeg`]、[`error`]。

pub mod budget;
pub mod error;
pub mod exec;
pub mod jpeg;
pub mod keyframes;
pub mod options;
pub mod plugin;
pub mod probe;
pub mod products;
pub mod scan;
pub mod select;
pub mod tools;
