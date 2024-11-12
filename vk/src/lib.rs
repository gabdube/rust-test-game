//! A very small vulkan wrapper for this game project
//! It only wraps the api the engine uses and shouldn't be used by anyone else
//! Do not use this outside of this project unless you hate yourself

#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc, clippy::missing_transmute_annotations)]

macro_rules! define_handle {
    ($name: ident, $ty: ident) => {
        #[repr(transparent)]
        #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash)]
        pub struct $name(*mut u8);
        impl Default for $name {
            fn default() -> $name {
                $name::null()
            }
        }

        unsafe impl Send for $name {}
        unsafe impl Sync for $name {}

        impl $name {
            pub const fn null() -> Self{
                $name(::std::ptr::null_mut())
            }
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                std::fmt::Debug::fmt(&self.0, f)
            }
        }
    }
}

macro_rules! define_nondispatchable_handle {
    ($ name : ident , $ ty : ident) => {
        #[repr(transparent)]
        #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Default)]
        pub struct $name(u64);
        impl $name {
            #[allow(dead_code)]
            pub const fn null() -> $name {
                $name(0)
            }

            pub fn is_null(&self) -> bool {
                *self == Self::null()
            }
        }
        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "0x{:x}", self.0)
            }
        }
    };
}

macro_rules! vk_enum {
    ($ name : ident) => {
        #[repr(transparent)]
        #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd)]
        pub struct $name(pub u32);
    }
}

macro_rules! vk_enum64 {
    ($ name : ident) => {
        #[repr(transparent)]
        #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd)]
        pub struct $name(pub u64);
    }
}

macro_rules! vk_bitflags {
    ($ name : ident) => {
        impl $name {
            #[inline]
            pub fn contains(self, other: $name) -> bool {
                self & other == other
            }

            #[inline]
            pub const fn bitor(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }
        }

        impl ::std::ops::BitAnd for $name {
            type Output = $name;
            #[inline]
            fn bitand(self, rhs: $name) -> $name {
                $name(self.0 & rhs.0)
            }
        }

        impl ::std::ops::BitOr for $name {
            type Output = $name;
            #[inline]
            fn bitor(self, rhs: $name) -> $name {
                $name(self.0 | rhs.0)
            }
        }
    }
}

mod vk100;
pub use vk100::*;

mod vk110;
pub use vk110::*;

mod vk120;
pub use vk120::*;

mod khr;
pub use khr::*;

mod ext;
pub use ext::*;

mod error;
pub use error::Error;

pub mod wrapper;
