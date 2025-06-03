use convert_case::*;
use std::fmt::*;

/// Generates C# code for a type.
pub struct DisplayCs<'a, T: DisplayBindings>(pub &'a T);

impl<'a, T: DisplayBindings> Display for DisplayCs<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.write_cs(f)
    }
}

/// Generates Rust code for a type.
pub struct DisplayRs<'a, T: DisplayBindings>(pub &'a T);

impl<'a, T: DisplayBindings> Display for DisplayRs<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.write_rs(f)
    }
}

/// A binding type that can generate either Rust or C# code.
pub trait DisplayBindings {
    /// Generates the C#-side code for this binding.
    fn write_cs(&self, f: &mut Formatter) -> Result;

    /// Generates the Rust-side code for this binding.
    fn write_rs(&self, f: &mut Formatter) -> Result;
}

/// A primitive type that can be shared between C# and Rust.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrimitiveType {
    /// The [`bool`] type.
    Bool,
    /// The [`u8`] type.
    U8,
    /// The [`u16`] type.
    U16,
    /// The [`u32`] type.
    U32,
    /// The [`u64`] type.
    U64,
    /// The [`i8`] type.
    I8,
    /// The [`i16`] type.
    I16,
    /// The [`i32`] type.
    I32,
    /// The [`i64`] type.
    I64,
    /// The [`f32`] type.
    F32,
    /// The [`f64`] type.
    F64,
    /// The [`String`] or [`str`] types.
    String
}

impl DisplayBindings for PrimitiveType {
    fn write_cs(&self, f: &mut Formatter) -> Result {
        f.write_str(match self {
            PrimitiveType::Bool => "bool",
            PrimitiveType::U8 => "byte",
            PrimitiveType::U16 => "ushort",
            PrimitiveType::U32 => "uint",
            PrimitiveType::U64 => "ulong",
            PrimitiveType::I8 => "sbyte",
            PrimitiveType::I16 => "short",
            PrimitiveType::I32 => "int",
            PrimitiveType::I64 => "long",
            PrimitiveType::F32 => "float",
            PrimitiveType::F64 => "double",
            PrimitiveType::String => "string",
        })
    }

    fn write_rs(&self, f: &mut Formatter) -> Result {
        f.write_str(match self {
            PrimitiveType::Bool => "bool",
            PrimitiveType::U8 => "u8",
            PrimitiveType::U16 => "u16",
            PrimitiveType::U32 => "u32",
            PrimitiveType::U64 => "u64",
            PrimitiveType::I8 => "i8",
            PrimitiveType::I16 => "i16",
            PrimitiveType::I32 => "i32",
            PrimitiveType::I64 => "i64",
            PrimitiveType::F32 => "f32",
            PrimitiveType::F64 => "f64",
            PrimitiveType::String => "VxString",
        })
    }
}

/// Defines the data necessary to use or marshal another type.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeReference {
    /// The type is externally-provided.
    Primitive(PrimitiveType)
}

impl DisplayBindings for TypeReference {
    fn write_cs(&self, f: &mut Formatter) -> Result {
        match self {
            TypeReference::Primitive(primitive_type) => primitive_type.write_cs(f),
        }
    }

    fn write_rs(&self, f: &mut Formatter) -> Result {
        match self {
            TypeReference::Primitive(primitive_type) => primitive_type.write_rs(f),
        }
    }
}

/// A top-level type definition.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item {
    /// A simple enum without any payload data.
    Enum {
        /// The name of the type.
        name: String,
        /// The possible enum values.
        variants: Vec<EnumVariant>,
        /// The doc-comment to include.
        docs: String,
    },
    /// A heap-allocated object backed by a handle.
    Class {
        /// The name of the type.
        name: String,
        /// The doc-comment to include.
        docs: String,
    },
    /// A plain-old-data type that can be copied from C# to Rust or vice-versa.
    Struct {
        /// The name of the type.
        name: String,
        /// The possible struct fields.
        fields: Vec<StructField>,
        /// Whether the struct implements [`Default`] on the Rust side.
        has_default: bool,
        /// The doc-comment to include.
        docs: String
    }
}

impl Item {
    /// Gets the doc-comment associated with this item.
    pub fn docs(&self) -> &str {
        match self {
            Item::Enum { docs, .. } => docs,
            Item::Class { docs, .. } => docs,
            Item::Struct { docs, .. } => docs
        }
    }

    /// The original name of the type.
    pub fn name(&self) -> &str {
        match self {
            Item::Enum { name, .. } => name,
            Item::Class { name, .. } => name,
            Item::Struct { name, .. } => name
        }
    }

    /// Gets the modified type name for the public C# API.
    pub fn cs_name(&self) -> String {
        self.name().to_string()
    }

    /// Gets the modified type name for C FFI.
    pub fn rs_name(&self) -> String {
        "Vx".to_string() + self.name()
    }

    /// Gets the modified type name that will be inserted before C FFI functions.
    pub fn rs_fn_name(&self) -> String {
        self.name().to_case(Case::Snake)
    }

    /// Creates the default field for a struct type in C#.
    fn write_cs_struct_default(&self, f: &mut Formatter) -> Result {
        write_cs_docs(f, "Returns the \"default value\" for a type.")?;
        f.write_fmt(format_args!("public static readonly {} Default = ({})Vx.{}_default();\n", self.cs_name(), self.cs_name(), self.rs_fn_name()))?;
        Ok(())
    }

    /// Creates the C#-side destructor for this type, assuming that it is a handle.
    fn write_cs_destructor(&self, f: &mut Formatter) -> Result {
        f.write_str("/// <inheritdoc/>\n")?;
        f.write_str("protected override void Free(VxObject* pointer) {\n")?;
        f.write_fmt(format_args!("    Vx.gui_{}_drop(pointer);\n", self.rs_fn_name()))?;
        f.write_str("}\n")?;
        Ok(())
    }

    /// Creates the default field initializer for a struct type in Rust.
    fn write_rs_struct_default(&self, f: &mut Formatter) -> Result {
        write_rs_docs(f, "Returns the \"default value\" for a type.")?;
        f.write_str("#[no_mangle]\n")?;
        f.write_fmt(format_args!("pub fn vx_{}_default() -> {} {{\n", self.rs_fn_name(), self.rs_name()))?;
        f.write_fmt(format_args!("    let value = {}::default();\n", self.name()))?;
        f.write_fmt(format_args!("    {} {{\n", self.rs_name()))?;

        let Self::Struct { fields, .. } = self else { panic!("Item was not struct") };
        for field in fields {
            f.write_fmt(format_args!("        {}: value.{}.into(),\n", field.rs_name(), field.name))?;
        }

        f.write_str("    }\n")?;
        f.write_str("}\n")?;
        Ok(())
    }

    /// Creates the Rust-side destructor for this type, assuming that it is a handle.
    fn write_rs_destructor(&self, f: &mut Formatter) -> Result {
        f.write_str("/// Frees the provided object.\n")?;
        f.write_str("///\n")?;
        f.write_str("/// # Safety\n")?;
        f.write_str("///\n")?;
        f.write_str("/// For this call to be sound, the pointer must refer to a live object of the corret type.\n");
        f.write_str("#[no_mangle]\n")?;
        f.write_fmt(format_args!("pub unsafe extern \"C\" fn vx_gui_{}_drop(value: *mut VxObject<{}>) {{\n)",
            self.rs_fn_name(), self.name()))?;
        f.write_str("    VxHandle::from_heap(value);\n")?;
        f.write_str("}}\n")?;
        Ok(())
    }
}

impl DisplayBindings for Item {
    fn write_cs(&self, f: &mut Formatter) -> Result {
        write_cs_docs(f, self.docs())?;
        match self {
            Item::Enum { variants, .. } => {
                f.write_fmt(format_args!("public enum {} {{\n", self.cs_name()))?;

                let mut members = String::new();
                for variant in variants {
                    write!(&mut members, "{}\n", DisplayCs(variant))?;
                }
                f.write_str(&indent(&members))?;

                f.write_str("}\n")?;
            },
            Item::Class { .. } => {
                f.write_fmt(format_args!("public unsafe sealed class {} : VxHandle {{\n", self.cs_name()))?;
                
                let mut destructor = String::new();
                self.write_cs_destructor(&mut Formatter::new(&mut destructor, f.options()))?;
                f.write_str(&indent(&destructor))?;

                f.write_str("}\n")?;
            },
            Item::Struct { fields, has_default, .. } => {
                f.write_fmt(format_args!("public unsafe struct {} {{\n", self.cs_name()))?;
                
                if *has_default {
                    let mut default = String::new();
                    self.write_cs_struct_default(&mut Formatter::new(&mut default, f.options()))?;
                    f.write_str(&indent(&default))?;
                    f.write_str("\n");
                }

                let mut members = String::new();
                for field in fields {
                    write!(&mut members, "{}\n", DisplayCs(field))?;
                }
                f.write_str(&indent(&members))?;

                f.write_str("}\n")?;
            }
        }
        Ok(())
    }

    fn write_rs(&self, f: &mut Formatter) -> Result {
        match self {
            Item::Enum { variants, .. } => {
                write_rs_docs(f, self.docs())?;
                f.write_str("#[derive(Copy, Clone)]]\n")?;
                f.write_str("#[repr(C)]\n")?;
                f.write_fmt(format_args!("pub enum {} {{\n", self.rs_name()))?;
                
                let mut members = String::new();
                for variant in variants {
                    write!(&mut members, "{}\n", DisplayRs(variant))?;
                }
                f.write_str(&indent(&members))?;

                f.write_str("}\n")?;
            },
            Item::Class { .. } => {
                self.write_rs_destructor(f);
            },
            Item::Struct { fields, has_default, .. } => {
                write_rs_docs(f, self.docs())?;
                f.write_str("#[derive(Copy, Clone)]]\n")?;
                f.write_str("#[repr(C)]\n")?;
                f.write_fmt(format_args!("pub struct {} {{\n", self.rs_name()))?;
                
                let mut members = String::new();
                for field in fields {
                    write!(&mut members, "{}\n", DisplayRs(field))?;
                }
                f.write_str(&indent(&members))?;

                f.write_str("}\n\n")?;

                if *has_default {
                    self.write_rs_struct_default(f)?;
                    f.write_str("\n")?;
                }
            }
        }
        Ok(())
    }
}

/// An enum variant.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EnumVariant {
    /// The name of the variant.
    pub name: String,
    /// The index of the variant, if any.
    pub index: Option<u64>,
    /// The doc-comment to include.
    pub docs: String
}

impl DisplayBindings for EnumVariant {
    fn write_cs(&self, f: &mut Formatter<'_>) -> Result {
        write_cs_docs(f, &self.docs)?;
        if let Some(index) = self.index {
            f.write_fmt(format_args!("{} = {index},", self.name))?;
        }
        else {
            f.write_fmt(format_args!("{},", self.name))?;
        }

        Ok(())
    }

    fn write_rs(&self, f: &mut Formatter) -> Result {
        write_rs_docs(f, &self.docs)?;
        if let Some(index) = self.index {
            f.write_fmt(format_args!("{} = {index},", self.name))?;
        }
        else {
            f.write_fmt(format_args!("{},", self.name))?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructField {
    /// The name of the field.
    pub name: String,
    /// The type of the field.
    pub ty: TypeReference,
    /// The doc-comment to include.
    pub docs: String
}

impl StructField {
    /// Gets the modified type name for the public C# API.
    pub fn cs_name(&self) -> String {
        self.name.to_case(Case::Pascal)
    }

    /// Gets the modified type name for C FFI.
    pub fn rs_name(&self) -> String {
        self.name.to_string()
    }
}

impl DisplayBindings for StructField {
    fn write_cs(&self, f: &mut Formatter) -> Result {
        write_cs_docs(f, &self.docs)?;
        f.write_fmt(format_args!("public {} {};\n", DisplayCs(&self.ty), self.cs_name()))
    }

    fn write_rs(&self, f: &mut Formatter) -> Result {
        write_rs_docs(f, &self.docs)?;
        f.write_fmt(format_args!("pub {}: {},", self.rs_name(), DisplayRs(&self.ty)))
    }
}

/// Adds one level of indentation (four spaces) to every line
/// of the string.
fn indent(value: &str) -> String {
    if value.is_empty() {
        String::new()
    }
    else {
        "    ".to_string() + &value.trim_end().replace("\n", "\n    ") + "\n"
    }
}

/// Writes a C# summary doc-comment.
fn write_cs_docs(f: &mut Formatter, docs: &str) -> Result {
    if !docs.is_empty() {
        f.write_str("/// <summary>\n")?;
        f.write_fmt(format_args!("/// {}\n", docs.trim_end().replace("\n", "\n/// ")))?;
        f.write_str("/// </summary>\n")?;
    }
    Ok(())
}

/// Writes a Rust doc-comment.
fn write_rs_docs(f: &mut Formatter, docs: &str) -> Result {
    if !docs.is_empty() {
        f.write_fmt(format_args!("/// {}\n", docs.trim_end().replace("\n", "\n/// ")))?;
    }

    Ok(())
}