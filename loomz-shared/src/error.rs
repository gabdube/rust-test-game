//! Common error model

#[derive(Debug)]
pub enum CommonErrorType {
    Undefined,
    Unimplemented,
    System,
    BackendInit,
    Synchronize,
    RenderRecord,
    RenderPresent,
}

impl ::std::fmt::Display for CommonErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            CommonErrorType::Undefined => "Undefined",
            CommonErrorType::Unimplemented => "Unimplemented",
            CommonErrorType::System => "System",
            CommonErrorType::BackendInit => "Backend initialization",
            CommonErrorType::Synchronize => "Gpu synchronisation",
            CommonErrorType::RenderRecord => "Rendering command recording",
            CommonErrorType::RenderPresent => "Rendering presentation",
        })
    }
}

#[derive(Debug)]
pub struct InnerCommonError {
    pub ty: CommonErrorType,
    pub line: u32,
    pub file: String,
    pub message: String,
    pub original: Option<Box<InnerCommonError>>,
}

impl ::std::fmt::Display for InnerCommonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(original) = self.original.as_ref() {
            writeln!(f, "[ERROR][{}:{}] {} - {}", self.file, self.line, self.ty, original)?
        }

        write!(f, "[ERROR][{}:{}] {} - {}", self.file, self.line, self.ty, self.message)
    }
}


#[derive(Debug)]
pub struct CommonError {
    pub inner: Box<InnerCommonError>
}

impl CommonError {
    #[cold]
    #[inline(never)]
    pub fn new(ty: CommonErrorType, file: &'static str, line: u32, message: String) -> Self {
        let inner = InnerCommonError {
            ty,
            file: file.to_string(),
            line,
            message,
            original: None,
        };

        CommonError {
            inner: Box::new(inner)
        }
    }

    #[cold]
    #[inline(never)]
    pub fn chain(self, ty: CommonErrorType, message: String, file: &'static str, line: u32) -> Self {
        let old = self.inner;
        let file = file.to_string();
        let inner = InnerCommonError {
            ty,
            line,
            file,
            message,
            original: Some(old),
        };

        CommonError { inner: Box::new(inner) }
    }
}

impl ::std::fmt::Display for CommonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[macro_export]
macro_rules! err {
    ($ty:expr, $($arg:tt)*) => {{
        let message = format!($($arg)*);
        $crate::CommonError::new($ty, file!(), line!(), message)
    }};
}

#[macro_export]
macro_rules! unimplemented_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::Unimplemented, $($arg)*) }; }

#[macro_export]
macro_rules! system_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::System, $($arg)*) }; }

#[macro_export]
macro_rules! backend_init_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::BackendInit, $($arg)*) } }

#[macro_export]
macro_rules! render_record_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::RenderRecord, $($arg)*) } }

#[macro_export]
macro_rules! synchronize_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::Synchronize, $($arg)*) } }

#[macro_export]
macro_rules! present_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::RenderPresent, $($arg)*) }; }
