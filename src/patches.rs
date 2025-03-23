use windows::{
    core::{Result as WindowsResult, PCWSTR},
    Win32::System::{
        LibraryLoader::GetModuleHandleW,
        Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS},
    },
};

use crate::config::Config;

pub fn place_all(config: &Config) -> WindowsResult<()> {
    let mut patch_helper = PatchHelper::new(config)?;

    // Graphics Main Heap:
    patch_helper.mul_u32(0xaef57c + 3, config.heap_sizes.graphics, true)?;

    // File Data Heap:
    patch_helper.mul_u32(0xaef59c + 3, config.heap_sizes.file_data, false)?;

    // Sound Sys Heap:
    patch_helper.mul_u32(0xaef5a3 + 4, config.heap_sizes.sound, true)?;

    // Network Heap:
    patch_helper.mul_u32(0xaef5ab + 3, config.heap_sizes.network, false)?;

    // String Heap:
    patch_helper.mul_u32(0xaef5b2 + 3, config.heap_sizes.string_data, false)?;

    // Temp Heap:
    patch_helper.mul_u32(0xaef5b9 + 3, config.heap_sizes.temp, true)?;

    // Temp2 Heap:
    patch_helper.mul_u32(0xaef5c0 + 3, config.heap_sizes.temp2, true)?;

    // Debug Heap:
    patch_helper.mul_u32(0xaef5c7 + 3, config.heap_sizes.debug, false)?;

    // Gui Default Heap:
    patch_helper.mul_u32(0xaef5ce + 4, config.heap_sizes.gui, false)?;

    // Regulation Heap:
    patch_helper.mul_u32(0x1c3512 + 2, config.heap_sizes.regulation, true)?;
    patch_helper.mul_u32(0x1c352e + 2, config.heap_sizes.regulation, false)?;

    // Menu Heap:
    patch_helper.mul_u32(0x1c357e + 2, config.heap_sizes.menu, true)?;
    patch_helper.mul_u32(0x1c359a + 2, config.heap_sizes.menu, false)?;

    // FaceGen Heap:
    patch_helper.mul_u32(0x1c35f3 + 2, config.heap_sizes.facegen, true)?;
    patch_helper.mul_u32(0x1c360f + 2, config.heap_sizes.facegen, false)?;

    // Player Heap:
    patch_helper.mul_u32(0x1c3670 + 2, config.heap_sizes.player, true)?;
    patch_helper.mul_u32(0x1c368c + 2, config.heap_sizes.player, false)?;

    // Sfx System Heap:
    patch_helper.mul_u32(0x1c372c + 2, config.heap_sizes.sfx, true)?;
    patch_helper.mul_u32(0x1c3748 + 2, config.heap_sizes.sfx, false)?;

    // Havok Heap:
    patch_helper.mul_u32(0x1c37a1 + 2, config.heap_sizes.havok, true)?;
    patch_helper.mul_u32(0x1c37c0 + 2, config.heap_sizes.havok, false)?;

    // SceneGraph Heap:
    patch_helper.mul_u32(0x1c3819 + 2, config.heap_sizes.scene_graph, true)?;
    patch_helper.mul_u32(0x1c3835 + 2, config.heap_sizes.scene_graph, false)?;

    // Morpheme Heap:
    patch_helper.mul_u32(0x1c388e + 2, config.heap_sizes.morpheme, true)?;
    patch_helper.mul_u32(0x1c38aa + 2, config.heap_sizes.morpheme, false)?;

    // Global Heap:
    patch_helper.set_global_heap_u32(0xaef595 + 3)?;

    // Morpheme fixed size vector expansion:
    const MORPHEME_DATA_FIXED_COUNT: u32 = 0x3000;
    const MORPHEME_DATA_ELEMENT_SIZE: u32 = 0x28;
    const MORPHEME_DATA_HEADER_SIZE: u32 = 0x28;

    let morpheme_data_new_count = MORPHEME_DATA_FIXED_COUNT
        .saturating_mul(config.heap_size_multiplier.max(1))
        .saturating_mul(config.heap_sizes.morpheme.max(1));

    patch_helper.set_u32(0x5f4f38 + 2, morpheme_data_new_count)?;

    let morpheme_data_total_size = MORPHEME_DATA_ELEMENT_SIZE
        .saturating_mul(morpheme_data_new_count)
        .saturating_add(MORPHEME_DATA_HEADER_SIZE);

    patch_helper.set_u32(0x5f4ef2 + 1, morpheme_data_total_size)?;
    patch_helper.set_u32(0x5f4f43 + 5, morpheme_data_total_size)?;

    Ok(())
}

struct PatchHelper<'a> {
    config: &'a Config,
    base_addr: usize,
    global_heap_bonus: u32,
}

impl<'a> PatchHelper<'a> {
    fn new(config: &'a Config) -> WindowsResult<PatchHelper<'a>> {
        unsafe {
            GetModuleHandleW(PCWSTR::null()).map(|h| Self {
                config,
                base_addr: h.0 as usize,
                global_heap_bonus: 0,
            })
        }
    }

    fn set_u32(&mut self, offset: usize, val: u32) -> WindowsResult<()> {
        Self::set_rwe_memory_u32(self.base_addr + offset)?;

        unsafe {
            let ptr = (self.base_addr + offset) as *mut u32;

            ptr.write_unaligned(val);
        }

        Ok(())
    }

    fn mul_u32(&mut self, offset: usize, val: u32, add_to_global_heap: bool) -> WindowsResult<()> {
        Self::set_rwe_memory_u32(self.base_addr + offset)?;

        let val = val
            .max(1)
            .saturating_mul(self.config.heap_size_multiplier.max(1));

        unsafe {
            let ptr = (self.base_addr + offset) as *mut u32;

            let base = ptr.read_unaligned();

            if add_to_global_heap {
                self.global_heap_bonus = self
                    .global_heap_bonus
                    .saturating_add(base.saturating_mul(val - 1));
            }

            ptr.write_unaligned(base.saturating_mul(val));
        }

        Ok(())
    }

    fn set_global_heap_u32(&mut self, offset: usize) -> WindowsResult<()> {
        Self::set_rwe_memory_u32(self.base_addr + offset)?;

        unsafe {
            let ptr = (self.base_addr + offset) as *mut u32;

            let base = ptr.read_unaligned();

            let with_mul = base.saturating_mul(self.config.heap_size_multiplier.max(1));
            let with_add = base.saturating_add(self.global_heap_bonus);

            ptr.write_unaligned(with_mul.max(with_add));
        }

        Ok(())
    }

    fn set_rwe_memory_u32(addr: usize) -> WindowsResult<()> {
        unsafe {
            VirtualProtect(
                addr as _,
                4,
                PAGE_EXECUTE_READWRITE,
                &mut PAGE_PROTECTION_FLAGS::default(),
            )
        }
    }
}
