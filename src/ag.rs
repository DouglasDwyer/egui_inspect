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

/// A top-level type definition.
#[derive(Clone, Debug)]
pub enum Item {
    /// A simple enum without any payload data.
    Enum {
        /// The doc-comment to include.
        docs: String,
        /// The name of the member.
        name: String,
        /// The possible enum values.
        variants: Vec<EnumVariant>
    }
}

impl Item {
    /// Gets the doc-comment associated with this item.
    pub fn docs(&self) -> &str {
        match self {
            Item::Enum { docs, .. } => docs,
        }
    }
}

impl DisplayBindings for Item {
    fn write_cs(&self, f: &mut Formatter) -> Result {
        write_cs_docs(f, self.docs())?;
        match self {
            Item::Enum { docs, name, variants } => {
                f.write_fmt(format_args!("public enum {name} {{\n"))?;

                let mut members = String::new();
                for variant in variants {
                    write!(&mut members, "{}", DisplayCs(variant))?;
                }
                f.write_str(&ident(&members))?;

                f.write_str("}\n")?;
            },
        }
        Ok(())
    }

    fn write_rs(&self, f: &mut Formatter) -> Result {
        write_rs_docs(f, self.docs())?;
        match self {
            Item::Enum { docs, name, variants } => {
                f.write_str("#[derive(Copy, Clone)]]\n")?;
                f.write_str("#[repr(C)]]\n")?;
                f.write_fmt(format_args!("pub enum Vx{name} {{\n"))?;
                
                let mut members = String::new();
                for variant in variants {
                    write!(&mut members, "{}", DisplayRs(variant))?;
                }
                f.write_str(&ident(&members))?;

                f.write_str("}\n")?;
            },
        }
        Ok(())
    }
}

/// An enum variant.
#[derive(Clone, Debug)]
pub struct EnumVariant {
    /// The doc-comment to include.
    pub docs: String,
    /// The name of the variant.
    pub name: String,
    /// The index of the variant, if any.
    pub index: Option<u64>
}

impl DisplayBindings for EnumVariant {
    fn write_cs(&self, f: &mut Formatter<'_>) -> Result {
        write_cs_docs(f, &self.docs)?;
        if let Some(index) = self.index {
            f.write_fmt(format_args!("{} = {index},\n", self.name))?;
        }
        else {
            f.write_fmt(format_args!("{},\n", self.name))?;
        }

        Ok(())
    }

    fn write_rs(&self, f: &mut Formatter) -> Result {
        write_rs_docs(f, &self.docs)?;
        if let Some(index) = self.index {
            f.write_fmt(format_args!("{} = {index},\n", self.name))?;
        }
        else {
            f.write_fmt(format_args!("{},\n", self.name))?;
        }

        Ok(())
    }
}

/// Adds one level of identation (four spaces) to every line
/// of the string.
fn ident(value: &str) -> String {
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