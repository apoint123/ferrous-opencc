//! 导入依赖用的模块

use anyhow::{Context, Result};
use bincode::{Decode, Encode, config};
use fst::MapBuilder;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Arc;

// 包含共享的编译逻辑
include!("../compiler_logic.rs");
