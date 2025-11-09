use crate::block::BlockType;
use serde::{Deserialize, Serialize};

/// Represents a stack of items in the inventory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemStack {
    pub block_type: BlockType,
    pub count: u32,
}

impl ItemStack {
    pub fn new(block_type: BlockType, count: u32) -> Self {
        Self { block_type, count }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn max_stack_size(&self) -> u32 {
        64 // Standard Minecraft stack size
    }

    pub fn can_add(&self, amount: u32) -> bool {
        self.count + amount <= self.max_stack_size()
    }
}

/// Main inventory structure with toolbar (9 slots) and storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    /// Toolbar slots (0-8), displayed at bottom of screen
    pub toolbar: [Option<ItemStack>; 9],
    /// Main inventory storage (27 slots, 3 rows of 9)
    pub storage: [Option<ItemStack>; 27],
    /// Currently selected toolbar slot (0-8)
    pub selected_slot: usize,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            toolbar: [None; 9],
            storage: [None; 27],
            selected_slot: 0,
        }
    }

    /// Initialize with some starter blocks
    pub fn with_starter_items() -> Self {
        let mut inv = Self::new();
        // Give player some starter blocks
        inv.toolbar[0] = Some(ItemStack::new(BlockType::Dirt, 64));
        inv.toolbar[1] = Some(ItemStack::new(BlockType::Grass, 64));
        inv.toolbar[2] = Some(ItemStack::new(BlockType::Sand, 64));
        inv.toolbar[3] = Some(ItemStack::new(BlockType::Wood, 64));
        inv.toolbar[4] = Some(ItemStack::new(BlockType::Planks, 64));
        inv.toolbar[5] = Some(ItemStack::new(BlockType::Leaves, 64));
        inv.toolbar[6] = Some(ItemStack::new(BlockType::Glass, 64));
        inv.toolbar[7] = Some(ItemStack::new(BlockType::Stone, 64));
        inv
    }

    /// Get the currently selected item stack
    pub fn get_selected_item(&self) -> Option<&ItemStack> {
        self.toolbar[self.selected_slot].as_ref()
    }

    /// Get the currently selected item stack mutably
    pub fn get_selected_item_mut(&mut self) -> &mut Option<ItemStack> {
        &mut self.toolbar[self.selected_slot]
    }

    /// Get the block type in the selected slot
    pub fn get_selected_block(&self) -> Option<BlockType> {
        self.toolbar[self.selected_slot].as_ref().map(|s| s.block_type)
    }

    /// Select next toolbar slot
    pub fn next_slot(&mut self) {
        self.selected_slot = (self.selected_slot + 1) % 9;
    }

    /// Select previous toolbar slot
    pub fn prev_slot(&mut self) {
        self.selected_slot = if self.selected_slot == 0 { 8 } else { self.selected_slot - 1 };
    }

    /// Try to add an item to the inventory
    /// Returns true if item was added, false if inventory is full
    pub fn add_item(&mut self, block_type: BlockType, amount: u32) -> bool {
        if amount == 0 {
            return true;
        }

        let mut remaining = amount;

        // First, try to add to existing stacks in toolbar
        for slot in &mut self.toolbar {
            if let Some(stack) = slot {
                if stack.block_type == block_type && !stack.is_empty() {
                    let can_add = stack.max_stack_size() - stack.count;
                    let to_add = remaining.min(can_add);
                    stack.count += to_add;
                    remaining -= to_add;
                    if remaining == 0 {
                        return true;
                    }
                }
            }
        }

        // Then try existing stacks in storage
        for slot in &mut self.storage {
            if let Some(stack) = slot {
                if stack.block_type == block_type && !stack.is_empty() {
                    let can_add = stack.max_stack_size() - stack.count;
                    let to_add = remaining.min(can_add);
                    stack.count += to_add;
                    remaining -= to_add;
                    if remaining == 0 {
                        return true;
                    }
                }
            }
        }

        // Create new stacks in empty slots
        while remaining > 0 {
            let stack_size = remaining.min(64);
            
            // Try toolbar first
            if let Some(empty_slot) = self.toolbar.iter_mut().find(|slot| slot.is_none()) {
                *empty_slot = Some(ItemStack::new(block_type, stack_size));
                remaining -= stack_size;
                continue;
            }

            // Then try storage
            if let Some(empty_slot) = self.storage.iter_mut().find(|slot| slot.is_none()) {
                *empty_slot = Some(ItemStack::new(block_type, stack_size));
                remaining -= stack_size;
                continue;
            }

            // No space left
            return false;
        }

        true
    }

    /// Try to remove an item from the selected slot
    /// Returns true if item was removed, false if slot is empty
    pub fn remove_selected_item(&mut self, amount: u32) -> bool {
        if let Some(stack) = &mut self.toolbar[self.selected_slot] {
            if stack.count >= amount {
                stack.count -= amount;
                if stack.count == 0 {
                    self.toolbar[self.selected_slot] = None;
                }
                return true;
            }
        }
        false
    }

    /// Check if the selected slot has at least one item
    pub fn has_selected_item(&self) -> bool {
        self.toolbar[self.selected_slot].as_ref().map_or(false, |s| s.count > 0)
    }

    /// Move item from one slot to another
    /// Returns true if successful
    pub fn move_item(&mut self, from_toolbar: bool, from_idx: usize, to_toolbar: bool, to_idx: usize) -> bool {
        // Get source and destination arrays
        let (from_slot, to_slot) = if from_toolbar && to_toolbar {
            if from_idx >= 9 || to_idx >= 9 {
                return false;
            }
            let ptr = self.toolbar.as_mut_ptr();
            unsafe {
                (ptr.add(from_idx), ptr.add(to_idx))
            }
        } else if from_toolbar && !to_toolbar {
            if from_idx >= 9 || to_idx >= 27 {
                return false;
            }
            // Can't use split_at_mut because they're different arrays
            // We'll handle this differently
            let from_item = self.toolbar[from_idx].take();
            let to_item = self.storage[to_idx].take();
            
            self.toolbar[from_idx] = to_item;
            self.storage[to_idx] = from_item;
            return true;
        } else if !from_toolbar && to_toolbar {
            if from_idx >= 27 || to_idx >= 9 {
                return false;
            }
            let from_item = self.storage[from_idx].take();
            let to_item = self.toolbar[to_idx].take();
            
            self.storage[from_idx] = to_item;
            self.toolbar[to_idx] = from_item;
            return true;
        } else {
            if from_idx >= 27 || to_idx >= 27 {
                return false;
            }
            let ptr = self.storage.as_mut_ptr();
            unsafe {
                (ptr.add(from_idx), ptr.add(to_idx))
            }
        };

        unsafe {
            // Swap the two slots
            std::ptr::swap(from_slot, to_slot);
        }
        
        true
    }

    /// Get total number of a specific block type in inventory
    pub fn count_block_type(&self, block_type: BlockType) -> u32 {
        let mut total = 0;
        for slot in &self.toolbar {
            if let Some(stack) = slot {
                if stack.block_type == block_type {
                    total += stack.count;
                }
            }
        }
        for slot in &self.storage {
            if let Some(stack) = slot {
                if stack.block_type == block_type {
                    total += stack.count;
                }
            }
        }
        total
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}
