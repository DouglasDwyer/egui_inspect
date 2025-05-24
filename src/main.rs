#![allow(warnings)]

use std::collections::*;
use rustdoc_types::*;

pub struct BindgenContext {
    known_types: HashMap<String, KnownType>,
    krate: Crate,
    remaining_items: Vec<Id>,
    result: String,
    total_items: usize
}

impl BindgenContext {
    pub fn new() -> Self {
        let known_types = Self::default_known_types();
        let krate = serde_json::from_str::<Crate>(include_str!("egui.json")).expect("Failed to parse egui");
        let mut remaining_items = krate.index.values().filter(Self::item_relevant).map(|x| x.id).collect::<Vec<_>>();
        let total_items = remaining_items.len();
        let result = String::new();

        Self {
            known_types,
            krate,
            remaining_items,
            total_items,
            result
        }
    }
    
    pub fn generate(&mut self) {
        self.generate_primitive_enums();
        self.generate_primitive_structs();
    }

    fn generate_primitive_structs(&mut self) {
        let mut remaining = self.remaining_items.clone();
        loop {
            let len = remaining.len();

            remaining.retain(|x| {
                let item = &self.krate.index[x];
                match &item.inner {
                    ItemEnum::Struct(_) => !self.generate_primitive_struct(item.id),
                    _ => true
                }
            });
            
            if len == remaining.len() {
                break;
            }
        }
        self.remaining_items = remaining;
    }

    fn generate_primitive_struct(&mut self, id: Id) -> bool {
        let ItemEnum::Struct(x) = &self.krate.index[&id].inner else { unreachable!() };
        match &x.kind {
            StructKind::Plain { fields, has_stripped_fields } => if *has_stripped_fields {
                false
            }
            else {
                println!("gobere!");
                if fields.iter().all(|x| if let Some(kt) = self.known_types.get(&self.rust_name(*x)) {
                    kt.kind == TypeKind::Copy
                }
                else {
                    println!("Wanted {:?} copy", &self.krate.index[x]);
                    false
                }) {
                    self.known_types.insert(self.rust_name(id), KnownType {
                        cs_name: self.rust_name(id),
                        kind: TypeKind::Copy
                    });
                    true
                }
                else {
                    false
                }
            },
            _ => false
        }
    }

    fn generate_primitive_enums(&mut self) {
        let mut remaining = self.remaining_items.clone();
        remaining.retain(|x| {
            let item = &self.krate.index[x];
            match &item.inner {
                ItemEnum::Enum(x) => !self.generate_primitive_enum(item.id),
                _ => true
            }
        });
        self.remaining_items = remaining;
    }
    
    fn generate_primitive_enum(&mut self, id: Id) -> bool {
        let ItemEnum::Enum(x) = &self.krate.index[&id].inner else { unreachable!() };
        if self.is_primitive_enum(x) {
            let enum_ty = &self.krate.index[&id];
            let cs_name = enum_ty.name.as_ref().expect("Item did not have name").to_owned();
            Self::write_summary_doc(&enum_ty.docs, 0, &mut self.result);
            self.result += &format!("public enum {}\n{{\n", cs_name);
            for variant in &x.variants {
                let var = &self.krate.index[variant];
                Self::write_summary_doc(&var.docs, 4, &mut self.result);
                let ItemEnum::Variant(y) = &var.inner else { unreachable!() };
                if let Some(d) = &y.discriminant {
                    self.result += &format!("    {} = {},\n", var.name.as_ref().expect("Failed to get item name"), d.value);
                }
                else {
                    self.result += &format!("    {},\n", var.name.as_ref().expect("Failed to get item name"));
                }
            }
            self.result += &format!("}}\n\n");
    
            self.known_types.insert(self.rust_name(id), KnownType {
                cs_name,
                kind: TypeKind::Copy
            });

            true
        }
        else {
            false
        }
    }
    
    fn rust_name(&self, id: Id) -> String {
        self.krate.index[&id].name.as_deref().unwrap_or("").to_string()
    }

    /// Checks if the enum only has primitive variants.
    fn is_primitive_enum(&self, x: &Enum) -> bool {
        for variant in &x.variants {
            let ItemEnum::Variant(x) = &self.krate.index[variant].inner else { unreachable!() };
            if x.kind != VariantKind::Plain {
                return false;
            }
        }
    
        true
    }
    
    /// Whether this is an item for which we will generate code.
    fn item_relevant(x: &&Item) -> bool {
        match &x.inner {
            ItemEnum::Union(_)
            | ItemEnum::Struct(_)
            | ItemEnum::Enum(_)
            | ItemEnum::Function(_)
            | ItemEnum::TypeAlias(_)
            | ItemEnum::Constant { .. }
            | ItemEnum::Static(_)
            | ItemEnum::ExternType
            | ItemEnum::Macro(_)
            | ItemEnum::ProcMacro(_) => true,
            _ => false
        }
    }
    
    fn default_known_types() -> HashMap<String, KnownType> {
        [
            ("i32", KnownType::new("int", TypeKind::Copy)),
            ("u32", KnownType::new("uint", TypeKind::Copy)),
            ("f32", KnownType::new("float", TypeKind::Copy)),
            ("f64", KnownType::new("double", TypeKind::Copy)),
            ("egui::Pos2", KnownType::new("IVec2", TypeKind::Copy)),
            ("egui::Vec2", KnownType::new("IVec2", TypeKind::Copy)),
            ("egui::Vec2", KnownType::new("IVec2", TypeKind::Copy)),
        ].into_iter().map(|(a, b)| (a.to_owned(), b)).collect()
    }

    fn write_summary_doc(data: &Option<String>, indent: usize, result: &mut String) {
        if let Some(docs) = data {
            let indent_str = " ".repeat(indent);
            let edited_label = docs.replace("\n", &format!("\n{indent_str}/// "));
            *result += &format!("{indent_str}/// <summary>\n{indent_str}/// {edited_label}\n{indent_str}/// </summary>\n");
        }
    }
}

#[derive(Clone, Debug)]
struct KnownType {
    pub cs_name: String,
    pub kind: TypeKind
}

impl KnownType {
    pub fn new(cs_name: impl Into<String>, kind: TypeKind) -> Self {
        Self { cs_name: cs_name.into(), kind }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TypeKind {
    Copy,
    Opaque
}

pub fn main() {
    let mut ctx = BindgenContext::new();
    ctx.generate();
    //println!("{}", ctx.result);
    println!("{:?}", ctx.known_types);
    println!("{} / {} items", ctx.total_items - ctx.remaining_items.len(), ctx.total_items);
}