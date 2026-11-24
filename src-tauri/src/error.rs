use serde::ser::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误：{0}")]
    Database(#[from] sqlx::Error),

    #[error("文件系统错误：{0}")]
    Io(#[from] std::io::Error),

    #[error("资源不存在：{0}")]
    NotFound(String),

    #[error("参数校验失败：{0}")]
    Validation(String),

    #[error("内部错误：{0}")]
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
