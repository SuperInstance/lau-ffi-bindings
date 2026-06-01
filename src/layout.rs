use serde::{Serialize, Deserialize};

/// Description of a single field in a C-compatible struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLayout {
    pub name: String,
    pub type_name: String,
    pub size: usize,
    pub alignment: usize,
    pub offset: usize,
}

/// Computed layout of a C-compatible struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructLayout {
    pub name: String,
    pub fields: Vec<FieldLayout>,
    pub total_size: usize,
    pub alignment: usize,
    pub padding: usize,
}

/// Align `offset` up to the given alignment boundary.
pub fn align_to(offset: usize, alignment: usize) -> usize {
    if alignment == 0 {
        return offset;
    }
    (offset + alignment - 1) & !(alignment - 1)
}

/// Compute padding needed between `offset` and the next aligned position.
pub fn compute_padding(offset: usize, alignment: usize) -> usize {
    let aligned = align_to(offset, alignment);
    aligned - offset
}

/// Builder for computing struct layouts with proper C alignment and padding.
pub struct StructLayoutBuilder {
    name: String,
    fields: Vec<FieldLayout>,
    current_offset: usize,
    max_align: usize,
}

impl StructLayoutBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: Vec::new(),
            current_offset: 0,
            max_align: 1,
        }
    }

    /// Add a field with the given type info, size, and alignment.
    /// Automatically inserts padding for alignment.
    pub fn add_field(&mut self, name: &str, type_name: &str, size: usize, alignment: usize) {
        let _padding = compute_padding(self.current_offset, alignment);
        let offset = align_to(self.current_offset, alignment);
        self.fields.push(FieldLayout {
            name: name.to_string(),
            type_name: type_name.to_string(),
            size,
            alignment,
            offset,
        });
        self.current_offset = offset + size;
        if alignment > self.max_align {
            self.max_align = alignment;
        }
    }

    /// Finalize the layout, computing total size with trailing padding.
    pub fn build(self) -> StructLayout {
        let total_size = align_to(self.current_offset, self.max_align);
        let padding = total_size - self.fields.iter().map(|f| f.size).sum::<usize>();
        StructLayout {
            name: self.name,
            fields: self.fields,
            total_size,
            alignment: self.max_align,
            padding,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{size_of, align_of};

    // A repr(C) struct for layout testing
    #[repr(C)]
    struct TestStruct {
        a: u8,
        b: u32,
        c: u16,
    }

    #[test]
    fn test_align_to() {
        assert_eq!(align_to(0, 4), 0);
        assert_eq!(align_to(1, 4), 4);
        assert_eq!(align_to(4, 4), 4);
        assert_eq!(align_to(5, 8), 8);
        assert_eq!(align_to(8, 8), 8);
    }

    #[test]
    fn test_compute_padding() {
        assert_eq!(compute_padding(0, 4), 0);
        assert_eq!(compute_padding(1, 4), 3);
        assert_eq!(compute_padding(5, 8), 3);
    }

    #[test]
    fn test_struct_layout_matches_rust() {
        let mut builder = StructLayoutBuilder::new("TestStruct");
        builder.add_field("a", "u8", size_of::<u8>(), align_of::<u8>());
        builder.add_field("b", "u32", size_of::<u32>(), align_of::<u32>());
        builder.add_field("c", "u16", size_of::<u16>(), align_of::<u16>());
        let layout = builder.build();

        assert_eq!(layout.fields[0].offset, 0); // a at 0
        assert_eq!(layout.fields[1].offset, 4); // b at 4 (aligned to 4)
        assert_eq!(layout.fields[2].offset, 8); // c at 8
        assert_eq!(layout.total_size, size_of::<TestStruct>());
        assert_eq!(layout.alignment, align_of::<TestStruct>());
    }

    #[test]
    fn test_struct_layout_size() {
        assert_eq!(size_of::<TestStruct>(), 12);
    }

    #[test]
    fn test_empty_struct_layout() {
        let layout = StructLayoutBuilder::new("Empty").build();
        // No fields → size 0, alignment 1
        assert_eq!(layout.total_size, 0);
        assert_eq!(layout.fields.len(), 0);
        assert!(layout.padding == 0);
    }

    #[test]
    fn test_layout_all_basic_types() {
        // Verify sizes and alignments of basic types
        assert_eq!(size_of::<u8>(), 1);
        assert_eq!(size_of::<u16>(), 2);
        assert_eq!(size_of::<u32>(), 4);
        assert_eq!(size_of::<u64>(), 8);
        assert_eq!(size_of::<f32>(), 4);
        assert_eq!(size_of::<f64>(), 8);
    }

    #[test]
    fn test_layout_serde() {
        let mut builder = StructLayoutBuilder::new("SerdeTest");
        builder.add_field("x", "i32", 4, 4);
        let layout = builder.build();
        let json = serde_json::to_string(&layout).unwrap();
        let back: StructLayout = serde_json::from_str(&json).unwrap();
        assert_eq!(layout.total_size, back.total_size);
    }
}
