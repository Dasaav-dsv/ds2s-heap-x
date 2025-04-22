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
    patch_helper.patch_morpheme_limit()?;

    // Patch DLFixedVector containers limited to 32 character resource slots:
    patch_helper.patch_character_resource_limit()?;

    // Patch DLFixedVector container limited to 48 FMod soundbanks:
    patch_helper.patch_soundbank_limit()?;

    // Patch arbitrary 255 `EnemyGeneratorCtrl` limit:
    patch_helper.set_u32(0x40e7d8 + 2, 0)?;

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

            let with_mul = base.saturating_mul(self.config.heap_sizes.global);
            let with_add = base.saturating_add(self.global_heap_bonus);

            ptr.write_unaligned(with_mul.max(with_add));
        }

        Ok(())
    }

    fn patch_morpheme_limit(&mut self) -> WindowsResult<()> {
        const MORPHEME_DATA_FIXED_COUNT: u32 = 0x3000;
        const MORPHEME_DATA_ELEMENT_SIZE: u32 = 0x28;
        const MORPHEME_DATA_HEADER_SIZE: u32 = 0x28;

        let morpheme_data_new_count =
            MORPHEME_DATA_FIXED_COUNT.saturating_mul(self.config.heap_sizes.morpheme);

        self.set_u32(0x5f4f38 + 2, morpheme_data_new_count)?;

        let morpheme_data_total_size = MORPHEME_DATA_ELEMENT_SIZE
            .saturating_mul(morpheme_data_new_count)
            .saturating_add(MORPHEME_DATA_HEADER_SIZE);

        self.set_u32(0x5f4ef2 + 1, morpheme_data_total_size)?;
        self.set_u32(0x5f4f43 + 5, morpheme_data_total_size)?;

        Ok(())
    }

    fn patch_character_resource_limit(&mut self) -> WindowsResult<()> {
        if !self.config.patch_character_limit {
            return Ok(());
        }

        /*
           Patching the code accessing the struct below to increase how
           many character types can be loaded by the game at once:

           struct ResObjectHolder {
               DLAllocator* allocator;
               DLFixedVector<ChrResModelObject*, 32> model_res_objects;
               DLFixedVector<ChrResMorphemeObject*, 32> morpheme_res_objects;
               DLFixedVector<ChrResSoundObject*, 32> sound_res_objects;
               DLFixedVector<ChrResTimeActObject*, 32> tae_res_objects;
           };

           The default limit is 32. After more than 32 characters are loaded, no new character
           resources can be inserted, and character loading will never finish. This leads to
           missing enemies and infinite loading screens.

           The layout of a DLFixedVector is:

           template <typename T, size_t Capacity>
           struct DLFixedVector {
               std::byte buffer[sizeof(T) * Capacity + alignof(T)];
               size_t size;
           };

           Note that the elements inside the buffer are manually aligned, so extra space is needed
           (at most the alignment of T).

           Any code accessing the fixed vector is compiled with the capacity and overall structure
           size, so *everything* in accessing code needs to be patched to increase the capacity.
        */

        const DLALLOCATOR_BASE_SIZE: u32 = 8;
        const DLFIXEDVECTOR_ELEMENT_SIZE: u32 = 8;
        const DLFIXEDVECTOR_BASE_CAPACITY: u32 = 32;

        // Patch the capacity from 32 to 1024, should realistically be enough:
        const DLFIXEDVECTOR_NEW_CAPACITY: u32 = DLFIXEDVECTOR_BASE_CAPACITY * 32;

        // sizeof(T) * Capacity + alignof(T) + sizeof(size_t)
        const DLFIXEDVECTOR_NEW_SIZE: u32 =
            DLFIXEDVECTOR_ELEMENT_SIZE * DLFIXEDVECTOR_NEW_CAPACITY + 8 + 8;

        // Total patched structure size
        const RES_OBJECT_HOLDER_SIZE: u32 = DLALLOCATOR_BASE_SIZE + DLFIXEDVECTOR_NEW_SIZE * 4;

        // Offsets of each fixed vector in `ResObjectHolder`
        const DLFIXEDVECTOR_0_OFFSET: u32 = DLALLOCATOR_BASE_SIZE;
        const DLFIXEDVECTOR_1_OFFSET: u32 = DLFIXEDVECTOR_0_OFFSET + DLFIXEDVECTOR_NEW_SIZE;
        const DLFIXEDVECTOR_2_OFFSET: u32 = DLFIXEDVECTOR_1_OFFSET + DLFIXEDVECTOR_NEW_SIZE;
        const DLFIXEDVECTOR_3_OFFSET: u32 = DLFIXEDVECTOR_2_OFFSET + DLFIXEDVECTOR_NEW_SIZE;

        // Offset of `size` field in `DLFixedVector<T, N>`
        const DLFIXEDVECTOR_SIZE_OFFSET: u32 = DLFIXEDVECTOR_NEW_SIZE - 8;

        // Offsets of each fixed vector's `size` field in `ResObjectHolder`
        const DLFIXEDVECTOR_0_SIZE_OFFSET: u32 = DLALLOCATOR_BASE_SIZE + DLFIXEDVECTOR_SIZE_OFFSET;
        const DLFIXEDVECTOR_1_SIZE_OFFSET: u32 =
            DLFIXEDVECTOR_0_SIZE_OFFSET + DLFIXEDVECTOR_NEW_SIZE;
        const DLFIXEDVECTOR_2_SIZE_OFFSET: u32 =
            DLFIXEDVECTOR_1_SIZE_OFFSET + DLFIXEDVECTOR_NEW_SIZE;
        const DLFIXEDVECTOR_3_SIZE_OFFSET: u32 =
            DLFIXEDVECTOR_2_SIZE_OFFSET + DLFIXEDVECTOR_NEW_SIZE;

        // DarkSoulsII.exe+0x165c80:
        self.set_u32(0x165c85 + 3, DLFIXEDVECTOR_0_SIZE_OFFSET)?;
        self.set_u32(0x165c8c + 3, DLFIXEDVECTOR_1_SIZE_OFFSET)?;
        self.set_u32(0x165c93 + 3, DLFIXEDVECTOR_2_SIZE_OFFSET)?;
        self.set_u32(0x165c9a + 3, DLFIXEDVECTOR_3_SIZE_OFFSET)?;

        // DarkSoulsII.exe+0x166370:
        self.set_u32(0x1663ad + 3, DLFIXEDVECTOR_NEW_SIZE)?;
        self.set_u32(0x1663b7 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0x166402 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0x166419 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0x166470 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0x166492 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0x1664b8 + 3, DLFIXEDVECTOR_NEW_SIZE)?;
        self.set_u32(0x1664c3 + 3, DLFIXEDVECTOR_NEW_SIZE)?;

        // DarkSoulsII.exe+0x166560:
        self.set_u32(0x166567 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;

        // DarkSoulsII.exe+0x1665c0:
        self.set_u32(0x1665c0 + 3, DLFIXEDVECTOR_1_OFFSET)?;
        self.set_u32(0x1665ca + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;

        // DarkSoulsII.exe+0x166620:
        self.set_u32(0x166620 + 3, DLFIXEDVECTOR_1_OFFSET)?;
        self.set_u32(0x16662a + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;

        // DarkSoulsII.exe+0x166680:
        self.set_u32(0x166687 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;

        // DarkSoulsII.exe+0x1666e0:
        self.set_u32(0x1666e0 + 3, DLFIXEDVECTOR_3_OFFSET)?;
        self.set_u32(0x1666ea + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;

        // DarkSoulsII.exe+0x1671d0:
        self.set_u32(0x167279 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0x167289 + 3, DLFIXEDVECTOR_NEW_SIZE)?;
        self.set_u32(0x1673eb + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0x167432 + 3, DLFIXEDVECTOR_NEW_SIZE)?;
        self.set_u32(0x1674c0 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0x1674d0 + 3, DLFIXEDVECTOR_NEW_SIZE)?;
        self.set_u32(0x167540 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0x16755e + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0x167585 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0x167592 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0x1675ac + 3, DLFIXEDVECTOR_NEW_SIZE)?;

        // DarkSoulsII.exe+0x167660:
        self.set_u32(0x16766e + 3, DLFIXEDVECTOR_NEW_SIZE)?;
        self.set_u32(0x16767b + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;

        // DarkSoulsII.exe+0x167780:
        self.set_u32(0x1677a4 + 3, DLFIXEDVECTOR_NEW_SIZE)?;
        self.set_u32(0x1677b1 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0x1677e5 + 4, DLFIXEDVECTOR_NEW_SIZE)?;
        self.set_u32(0x16793d + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0x167951 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        // Neutralize size overflow check exceptions
        self.set_u32(0x1677ee + 2, 0)?;
        self.set_u32(0x167947, 0xF9909090)?;

        // DarkSoulsII.exe+0x1679c0:
        self.set_u32(0x1679ca + 3, DLFIXEDVECTOR_NEW_SIZE)?;
        self.set_u32(0x1679d7 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;

        // DarkSoulsII.exe+0x350e00:
        self.set_u32(0x350e16 + 1, RES_OBJECT_HOLDER_SIZE)?;

        Ok(())
    }

    fn patch_soundbank_limit(&mut self) -> WindowsResult<()> {
        if !self.config.patch_soundbank_limit {
            return Ok(());
        }

        /*
            Look above at `PatchHelper::patch_character_resource_limit` for detailed layout information.

            struct RegisteredBankHolder {
               uint32_t total_count;
               DLFixedVector<RegisteredBank, 48> registered_banks;
            };

            sizeof(RegisteredBank) == 632, alignof(RegisteredBank) == 8
        */

        const DLFIXEDVECTOR_ELEMENT_SIZE: u32 = 632;

        // More than the total number of all soundbanks in the /sound directory
        const DLFIXEDVECTOR_NEW_CAPACITY: u32 = 513;

        // sizeof(T) * Capacity + alignof(T) + sizeof(size_t)
        const DLFIXEDVECTOR_NEW_SIZE: u32 =
            DLFIXEDVECTOR_ELEMENT_SIZE * DLFIXEDVECTOR_NEW_CAPACITY + 8 + 8;

        // Total patched structure size
        const REGISTERED_BANK_HOLDER_SIZE: u32 = 8 + DLFIXEDVECTOR_NEW_SIZE;

        // Offset of `size` field in `DLFixedVector<T, N>`
        const DLFIXEDVECTOR_SIZE_OFFSET: u32 = DLFIXEDVECTOR_NEW_SIZE - 8;

        // Offsets of each fixed vector's `size` field in `ResObjectHolder`
        const DLFIXEDVECTOR_0_SIZE_OFFSET: u32 = 8 + DLFIXEDVECTOR_SIZE_OFFSET;

        // DarkSoulsII.exe+0xb074d0:
        self.set_u32(0xb07741 + 1, REGISTERED_BANK_HOLDER_SIZE)?;

        // DarkSoulsII.exe+0xb57df0:
        self.set_u32(0xb57dfa + 3, DLFIXEDVECTOR_0_SIZE_OFFSET)?;

        // DarkSoulsII.exe+0xb57d70:
        self.set_u32(0xb57d74 + 3, DLFIXEDVECTOR_0_SIZE_OFFSET)?;

        // DarkSoulsII.exe+0xb580f0:
        self.set_u32(0xb58113 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;

        // DarkSoulsII.exe+0xb58240:
        self.set_u32(0xb5825d, 0xF9909090)?;
        self.set_u32(0xb5825d + 4, 0x90909090)?;

        // DarkSoulsII.exe+0xb583a0:
        self.set_u32(0xb583c6 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0xb58521 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0xb58549 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0xb58575 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0xb5857c + 3, DLFIXEDVECTOR_0_SIZE_OFFSET)?;

        // DarkSoulsII.exe+0xb58650:
        self.set_u32(0xb58654 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;
        self.set_u32(0xb5865e, 0xF9909090)?;
        self.set_u32(0xb58667 + 3, DLFIXEDVECTOR_SIZE_OFFSET)?;

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
