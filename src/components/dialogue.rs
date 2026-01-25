//! 对话系统组件

use bevy::prelude::*;

/// 单行对话数据
#[derive(Debug, Clone)]
pub struct DialogueLine {
    /// 说话者名称
    pub speaker: String,
    /// 对话文本
    pub text: String,
}

impl DialogueLine {
    pub fn new(speaker: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            speaker: speaker.into(),
            text: text.into(),
        }
    }
}

/// 对话序列组件/资源
#[derive(Component, Resource, Debug, Clone)]
pub struct Dialogue {
    /// 所有台词
    pub lines: Vec<DialogueLine>,
    /// 当前播放索引
    pub index: usize,
    /// 是否已播放完毕
    finished: bool,
}

impl Dialogue {
    pub fn new(lines: Vec<DialogueLine>) -> Self {
        Self {
            lines,
            index: 0,
            finished: false,
        }
    }

    /// 获取当前行
    pub fn current_line(&self) -> Option<&DialogueLine> {
        self.lines.get(self.index)
    }

    /// 推进到下一行
    pub fn next(&mut self) {
        if self.index + 1 < self.lines.len() {
            self.index += 1;
        } else {
            self.finished = true;
        }
    }

    /// 是否结束
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}
