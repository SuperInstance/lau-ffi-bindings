use crate::layout::StructLayout;

/// Generates C header file declarations from Rust struct layouts.
pub struct HeaderGenerator {
    includes: Vec<String>,
    forward_declarations: Vec<String>,
    structs: Vec<String>,
    typedefs: Vec<String>,
    function_declarations: Vec<String>,
}

impl HeaderGenerator {
    pub fn new() -> Self {
        Self {
            includes: Vec::new(),
            forward_declarations: Vec::new(),
            structs: Vec::new(),
            typedefs: Vec::new(),
            function_declarations: Vec::new(),
        }
    }

    /// Add an #include directive.
    pub fn add_include(&mut self, header: &str) {
        self.includes.push(format!("#include <{}>", header));
    }

    /// Add a typedef.
    pub fn add_typedef(&mut self, c_type: &str, alias: &str) {
        self.typedefs.push(format!("typedef {} {};", c_type, alias));
    }

    /// Add a forward declaration for a struct.
    pub fn add_forward_declaration(&mut self, name: &str) {
        self.forward_declarations.push(format!("struct {};", name));
    }

    /// Generate a C struct definition from a StructLayout.
    pub fn add_struct(&mut self, layout: &StructLayout) {
        let mut lines = Vec::new();
        lines.push(format!("struct {} {{", layout.name));
        for field in &layout.fields {
            lines.push(format!("    {} {};", field.type_name, field.name));
        }
        lines.push("};".to_string());
        // Add static assertions for size and alignment
        lines.push(format!(
            "_Static_assert(sizeof(struct {}) == {}, \"size mismatch\");",
            layout.name, layout.total_size
        ));
        lines.push(format!(
            "_Static_assert(_Alignof(struct {}) == {}, \"alignment mismatch\");",
            layout.name, layout.alignment
        ));
        self.structs.push(lines.join("\n"));
    }

    /// Add a C function declaration.
    pub fn add_function(&mut self, ret_type: &str, name: &str, params: &[(String, String)]) {
        let param_str = if params.is_empty() {
            "void".to_string()
        } else {
            params
                .iter()
                .map(|(t, n)| format!("{} {}", t, n))
                .collect::<Vec<_>>()
                .join(", ")
        };
        self.function_declarations.push(format!("{} {}({});", ret_type, name, param_str));
    }

    /// Generate the complete header file content.
    pub fn generate(&self, guard: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!("#ifndef {}\n", guard));
        out.push_str(&format!("#define {}\n\n", guard));

        for inc in &self.includes {
            out.push_str(inc);
            out.push('\n');
        }
        if !self.includes.is_empty() {
            out.push('\n');
        }

        // extern C guard
        out.push_str("#ifdef __cplusplus\n");
        out.push_str("extern \"C\" {\n");
        out.push_str("#endif\n\n");

        for td in &self.typedefs {
            out.push_str(td);
            out.push('\n');
        }
        if !self.typedefs.is_empty() {
            out.push('\n');
        }

        for fd in &self.forward_declarations {
            out.push_str(fd);
            out.push('\n');
        }
        if !self.forward_declarations.is_empty() {
            out.push('\n');
        }

        for s in &self.structs {
            out.push_str(s);
            out.push_str("\n\n");
        }

        for f in &self.function_declarations {
            out.push_str(f);
            out.push('\n');
        }
        if !self.function_declarations.is_empty() {
            out.push('\n');
        }

        out.push_str("#ifdef __cplusplus\n");
        out.push_str("}\n");
        out.push_str("#endif\n\n");
        out.push_str(&format!("#endif /* {} */\n", guard));
        out
    }
}

impl Default for HeaderGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::StructLayoutBuilder;

    #[test]
    fn test_basic_header_generation() {
        let mut gen = HeaderGenerator::new();
        gen.add_include("stdint.h");
        gen.add_typedef("int32_t", "my_int");
        let content = gen.generate("TEST_H");
        assert!(content.contains("#include <stdint.h>"));
        assert!(content.contains("typedef int32_t my_int;"));
        assert!(content.contains("#ifndef TEST_H"));
        assert!(content.contains("#endif"));
        assert!(content.contains("extern \"C\""));
    }

    #[test]
    fn test_struct_header_generation() {
        let mut builder = StructLayoutBuilder::new("Point");
        builder.add_field("x", "int32_t", 4, 4);
        builder.add_field("y", "int32_t", 4, 4);
        let layout = builder.build();

        let mut gen = HeaderGenerator::new();
        gen.add_include("stdint.h");
        gen.add_struct(&layout);
        let content = gen.generate("POINT_H");

        assert!(content.contains("struct Point {"));
        assert!(content.contains("int32_t x;"));
        assert!(content.contains("int32_t y;"));
        assert!(content.contains("_Static_assert"));
    }

    #[test]
    fn test_function_declaration() {
        let mut gen = HeaderGenerator::new();
        gen.add_function("int32_t", "add", &[
            ("int32_t".to_string(), "a".to_string()),
            ("int32_t".to_string(), "b".to_string()),
        ]);
        let content = gen.generate("FUNC_H");
        assert!(content.contains("int32_t add(int32_t a, int32_t b);"));
    }

    #[test]
    fn test_void_params() {
        let mut gen = HeaderGenerator::new();
        gen.add_function("void", "init", &[]);
        let content = gen.generate("INIT_H");
        assert!(content.contains("void init(void);"));
    }

    #[test]
    fn test_forward_declaration() {
        let mut gen = HeaderGenerator::new();
        gen.add_forward_declaration("Agent");
        let content = gen.generate("AGENT_H");
        assert!(content.contains("struct Agent;"));
    }
}
