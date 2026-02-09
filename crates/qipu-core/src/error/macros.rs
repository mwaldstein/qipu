//! Error macros for qipu

/// Macro for creating invalid value errors
#[macro_export]
macro_rules! bail_invalid {
    ($context:expr, $value:expr) => {
        return Err($crate::error::QipuError::invalid_value($context, $value))
    };
}

/// Macro for creating usage errors
#[macro_export]
macro_rules! bail_usage {
    ($msg:expr) => {
        return Err($crate::error::QipuError::UsageError($msg.to_string()))
    };
}

/// Macro for creating unsupported errors
#[macro_export]
macro_rules! bail_unsupported {
    ($context:expr, $value:expr, $supported:expr) => {
        return Err($crate::error::QipuError::unsupported(
            $context, $value, $supported,
        ))
    };
}

/// Macro for mapping database errors
#[macro_export]
macro_rules! map_db_err {
    ($op:expr, $error:expr) => {
        $crate::error::QipuError::db_operation($op, $error)
    };
}
