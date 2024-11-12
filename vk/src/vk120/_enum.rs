use crate::StructureType;

impl StructureType {
    pub const PHYSICAL_DEVICE_TIMELINE_SEMAPHORE_FEATURES: Self = Self(1_000_207_000);
    pub const SEMAPHORE_TYPE_CREATE_INFO: Self = Self(1_000_207_002);
    pub const TIMELINE_SEMAPHORE_SUBMIT_INFO: Self = Self(1_000_207_003);
    pub const SEMAPHORE_WAIT_INFO: Self = Self(1_000_207_004);
}

vk_enum!(ResolveModeFlagsBits);
vk_bitflags!(ResolveModeFlagsBits);

impl ResolveModeFlagsBits {
    pub const NONE: Self = Self(0x000);
    pub const SAMPLE_ZERO: Self = Self(0x001);
    pub const AVERAGE: Self = Self(0x002);
    pub const MIN: Self = Self(0x004);
    pub const MAX: Self = Self(0x008);
}

vk_enum!(SemaphoreWaitFlagsBits);
vk_bitflags!(SemaphoreWaitFlagsBits);

impl SemaphoreWaitFlagsBits {
    pub const ANY_BIT: Self = Self(0x1);
}

vk_enum!(SemaphoreType);
impl SemaphoreType {
    pub const BINARY: Self = Self(0);
    pub const TIMELINE: Self = Self(0x001);
}
