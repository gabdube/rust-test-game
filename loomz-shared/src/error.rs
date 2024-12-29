//! Common error model

#[derive(Debug)]
pub enum CommonErrorType {
    Undefined,
    Unimplemented,
    System,
    Assets,
    Api,
    BackendInit,
    BackendGeneric,
    Synchronize,
    RenderRecord,
    RenderPresent,
    SaveLoad,
}

impl ::std::fmt::Display for CommonErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            CommonErrorType::Undefined => "Undefined",
            CommonErrorType::Unimplemented => "Unimplemented",
            CommonErrorType::System => "System",
            CommonErrorType::Assets => "Assets",
            CommonErrorType::Api => "Api",
            CommonErrorType::BackendInit => "Backend initialization",
            CommonErrorType::BackendGeneric => "Backend generic error",
            CommonErrorType::Synchronize => "Gpu synchronisation",
            CommonErrorType::RenderRecord => "Rendering command recording",
            CommonErrorType::RenderPresent => "Rendering presentation",
            CommonErrorType::SaveLoad => "Save & Load",
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
            writeln!(f, "{}", original)?
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

    pub fn merge(&mut self, mut other: Self) {
        ::std::mem::swap(self, &mut other);
        self.inner.original = Some(other.inner);
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
macro_rules! chain_err {
    ($err:expr, $ty:expr, $($arg:tt)*) => {{
        let message = format!($($arg)*);
        let file = file!();
        $err.chain($ty, message, file, line!())
    }};

}

#[macro_export]
macro_rules! undefined_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::Undefined, $($arg)*) }; }

#[macro_export]
macro_rules! unimplemented_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::Unimplemented, $($arg)*) }; }

#[macro_export]
macro_rules! system_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::System, $($arg)*) }; }

#[macro_export]
macro_rules! api_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::Api, $($arg)*) }; }

#[macro_export]
macro_rules! assets_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::Assets, $($arg)*) }; }

#[macro_export]
macro_rules! backend_init_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::BackendInit, $($arg)*) } }

#[macro_export]
macro_rules! backend_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::BackendGeneric, $($arg)*) } }

#[macro_export]
macro_rules! render_record_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::RenderRecord, $($arg)*) } }

#[macro_export]
macro_rules! synchronize_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::Synchronize, $($arg)*) } }

#[macro_export]
macro_rules! present_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::RenderPresent, $($arg)*) }; }

#[macro_export]
macro_rules! save_err { ($($arg:tt)*) => { $crate::err!($crate::CommonErrorType::SaveLoad, $($arg)*) }; }
