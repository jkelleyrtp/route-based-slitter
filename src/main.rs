use std::collections::{HashMap, HashSet};

use dioxus::prelude::{dioxus_elements::mo, dioxus_router::routable, *};
use failure::ResultExt;
use log::LevelFilter;
use rayon::prelude::*;
use walrus::{ir::VisitorMut, ImportKind, InstrLocId, LocalId, Module};

// todo: I want to make a lil explorer tool that lets me mess with the wasm binary
//
// Scripting is fun but I think we can get pretty far with a lil ui
fn main() {
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! { "hello world" }
}

static CONTENTS: &[u8] = include_bytes!("../docsite_bg.wasm");
const ROUTES: [&str; 18] = [
    "Homepage",
    "Awesome",
    "Deploy",
    "Tutorial",
    "BlogList",
    "PostRelease050",
    "PostTemplate",
    "PostFulltime",
    "PostRelease040",
    "PostRelease030",
    "PostRelease020",
    "PostRelease010",
    "Learn",
    "Docs",
    "DocsO3",
    "DocsO4",
    "Docs",
    "Err404",
];
#[test]
fn snip_one() {
    let out = snip_route(CONTENTS, vec!["Homepage".to_string()]);
    print_saved(out.len(), CONTENTS.len(), "Homepage");
}

/// Split the wasm binary, cutting out *all* but one function at a time.
#[test]
fn properly_split() {
    for route in ROUTES {
        let isolated = ROUTES
            .iter()
            .filter(|f| **f != route)
            .map(|f| f.to_string())
            .collect::<Vec<_>>();

        let out = snip_route(CONTENTS, isolated);
        print_saved(out.len(), CONTENTS.len(), route);
    }
}

#[test]
fn load_binary() {
    let initial_bytes = CONTENTS.len();
    println!("{} bytes", initial_bytes);

    let mut saved = 0;

    // Do a per-route split
    for route in ROUTES {
        let out = snip_route(CONTENTS, vec![route.to_string()]);
        print_saved(out.len(), initial_bytes, route);
        saved += (initial_bytes - out.len());
    }

    // Now do a split with every route enabled
    let out = snip_route(CONTENTS, ROUTES.iter().map(|f| f.to_string()).collect());

    // print_saved(saved, initial_bytes, "all-individual");
    println!("Saved {} bytes", saved);
    print_saved(out.len(), initial_bytes, "all")

    // std::fs::write("docsite_out.wasm", &o);

    // include_bytes!("../harnesses/web-harness/dist/assets/dioxus/web-harness_bg.wasm");
    // include_bytes!("../harnesses/web-harness/dist/assets/dioxus/web-harness_bg.wasm");
}

fn print_saved(out_len: usize, initial_bytes: usize, route: &str) {
    // println!("{} bytes", out_bytes);
    let saved = initial_bytes - out_len;
    println!(
        "{route}: {} bytes, or {:.3} %",
        saved,
        (saved as f32 / initial_bytes as f32) * 100.0
    );
}

// Returns the number of bytes saved by snipping this route
fn snip_route(
    // The wasm binary
    contents: &[u8],

    // The route name
    names: Vec<String>,
) -> Vec<u8> {
    let mut config = walrus::ModuleConfig::new();

    let mut module = config.parse(contents).unwrap();

    // let props_of_functions = names.iter().map(|name| format!("{name}Props")).collect();

    let opts = Options {
        functions: names,
        // patterns: props_of_functions,
        ..Options::default()
    };

    snip(&mut module, opts);

    // let output = "output.wasm";
    let out = module.emit_wasm();

    out
}

/// Options for controlling which functions in what `.wasm` file should be
/// snipped.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// The functions that should be snipped from the `.wasm` file.
    pub functions: Vec<String>,

    /// The regex patterns whose matches should be snipped from the `.wasm`
    /// file.
    pub patterns: Vec<String>,

    /// Should Rust `std::fmt` and `core::fmt` functions be snipped?
    pub snip_rust_fmt_code: bool,

    /// Should Rust `std::panicking` and `core::panicking` functions be snipped?
    pub snip_rust_panicking_code: bool,

    /// Should we skip generating [the "producers" custom
    /// section](https://github.com/WebAssembly/tool-conventions/blob/master/ProducersSection.md)?
    pub skip_producers_section: bool,
}

/// Snip the functions from the input file described by the options.
pub fn snip(module: &mut walrus::Module, options: Options) -> Result<(), failure::Error> {
    if !options.skip_producers_section {
        module
            .producers
            .add_processed_by("wasm-snip", env!("CARGO_PKG_VERSION"));
    }

    let names: HashSet<String> = options.functions.iter().cloned().collect();
    let re_set = build_regex_set(options).context("failed to compile regex")?;
    let to_snip = find_functions_to_snip(&module, &names, &re_set);

    // println!("Found functions {:?}", to_snip);

    replace_calls_with_unreachable(module, &to_snip);
    unexport_snipped_functions(module, &to_snip);
    unimport_snipped_functions(module, &to_snip);
    snip_table_elements(module, &to_snip);
    delete_functions_to_snip(module, &to_snip);
    walrus::passes::gc::run(module);

    Ok(())
}

fn build_regex_set(mut options: Options) -> Result<regex::RegexSet, failure::Error> {
    // Snip the Rust `fmt` code, if requested.
    if options.snip_rust_fmt_code {
        // Mangled symbols.
        options.patterns.push(".*4core3fmt.*".into());
        options.patterns.push(".*3std3fmt.*".into());

        // Mangled in impl.
        options.patterns.push(r#".*core\.\.fmt\.\..*"#.into());
        options.patterns.push(r#".*std\.\.fmt\.\..*"#.into());

        // Demangled symbols.
        options.patterns.push(".*core::fmt::.*".into());
        options.patterns.push(".*std::fmt::.*".into());
    }

    // Snip the Rust `panicking` code, if requested.
    if options.snip_rust_panicking_code {
        // Mangled symbols.
        options.patterns.push(".*4core9panicking.*".into());
        options.patterns.push(".*3std9panicking.*".into());

        // Mangled in impl.
        options.patterns.push(r#".*core\.\.panicking\.\..*"#.into());
        options.patterns.push(r#".*std\.\.panicking\.\..*"#.into());

        // Demangled symbols.
        options.patterns.push(".*core::panicking::.*".into());
        options.patterns.push(".*std::panicking::.*".into());
    }

    Ok(regex::RegexSet::new(options.patterns)?)
}

fn find_functions_to_snip(
    module: &walrus::Module,
    names: &HashSet<String>,
    re_set: &regex::RegexSet,
) -> HashSet<walrus::FunctionId> {
    module
        .funcs
        .par_iter()
        .filter_map(|f| {
            f.name.as_ref().and_then(|name| {
                if names.contains(name) || re_set.is_match(name) {
                    Some(f.id())
                } else {
                    None
                }
            })
        })
        .collect()
}

fn delete_functions_to_snip(module: &mut walrus::Module, to_snip: &HashSet<walrus::FunctionId>) {
    for f in to_snip.iter().cloned() {
        module.funcs.delete(f);
    }
}

fn replace_calls_with_unreachable(
    module: &mut walrus::Module,
    to_snip: &HashSet<walrus::FunctionId>,
) {
    struct Replacer<'a> {
        to_snip: &'a HashSet<walrus::FunctionId>,
    }

    impl Replacer<'_> {
        fn should_snip_call(&self, instr: &walrus::ir::Instr) -> bool {
            if let walrus::ir::Instr::Call(walrus::ir::Call { func }) = instr {
                if self.to_snip.contains(func) {
                    return true;
                }
            }
            false
        }
    }

    impl VisitorMut for Replacer<'_> {
        fn visit_instr_mut(&mut self, instr: &mut walrus::ir::Instr, id: &mut InstrLocId) {
            if self.should_snip_call(instr) {
                *instr = walrus::ir::Unreachable {}.into();
            }
        }
    }

    module.funcs.par_iter_local_mut().for_each(|(id, func)| {
        // Don't bother transforming functions that we are snipping.
        if to_snip.contains(&id) {
            return;
        }

        let entry = func.entry_block();
        walrus::ir::dfs_pre_order_mut(&mut Replacer { to_snip }, func, entry);
    });
}

fn unexport_snipped_functions(module: &mut walrus::Module, to_snip: &HashSet<walrus::FunctionId>) {
    let exports_to_snip: HashSet<walrus::ExportId> = module
        .exports
        .iter()
        .filter_map(|e| match e.item {
            walrus::ExportItem::Function(f) if to_snip.contains(&f) => Some(e.id()),
            _ => None,
        })
        .collect();

    for e in exports_to_snip {
        module.exports.delete(e);
    }
}

fn unimport_snipped_functions(module: &mut walrus::Module, to_snip: &HashSet<walrus::FunctionId>) {
    let imports_to_snip: HashSet<walrus::ImportId> = module
        .imports
        .iter()
        .filter_map(|i| match i.kind {
            walrus::ImportKind::Function(f) if to_snip.contains(&f) => Some(i.id()),
            _ => None,
        })
        .collect();

    for i in imports_to_snip {
        module.imports.delete(i);
    }
}

fn snip_table_elements(module: &mut walrus::Module, to_snip: &HashSet<walrus::FunctionId>) {
    let mut unreachable_funcs: HashMap<walrus::TypeId, walrus::FunctionId> = Default::default();

    let make_unreachable_func = |ty: walrus::TypeId,
                                 types: &mut walrus::ModuleTypes,
                                 locals: &mut walrus::ModuleLocals,
                                 funcs: &mut walrus::ModuleFunctions|
     -> walrus::FunctionId {
        let ty = types.get(ty);
        let params = ty.params().to_vec();
        let locals: Vec<_> = params.iter().map(|ty| locals.add(*ty)).collect();
        let results = ty.results().to_vec();
        let mut builder = walrus::FunctionBuilder::new(types, &params, &results);
        builder.func_body().unreachable();
        builder.finish(locals, funcs)
    };

    // println!("Looking for functions {:?}", to_snip);

    for t in module.tables.iter_mut() {
        // println!("table {:?}", t);

        if let walrus::ValType::Funcref = t.element_ty {
            let types = &mut module.types;
            let locals = &mut module.locals;
            let funcs = &mut module.funcs;

            // println!("Function table {:?}", t.name);
            // println!("Function table {:?}", t);

            for snip in to_snip {
                // if t.elem_segments.contains(snip) {}
            }

            // t.elem_segments
            //     .iter_mut()
            //     .flat_map(|el| el)
            //     .filter(|f| to_snip.contains(f))
            //     .for_each(|el| {
            //         let ty = funcs.get(*el).ty();
            //         *el = *unreachable_funcs
            //             .entry(ty)
            //             .or_insert_with(|| make_unreachable_func(ty, types, locals, funcs));
            //     });
            // ft.elements
            //     .iter_mut()
            //     .flat_map(|el| el)
            //     .filter(|f| to_snip.contains(f))
            //     .for_each(|el| {
            //         let ty = funcs.get(*el).ty();
            //         *el = *unreachable_funcs
            //             .entry(ty)
            //             .or_insert_with(|| make_unreachable_func(ty, types, locals, funcs));
            //     });

            // ft.relative_elements
            //     .iter_mut()
            //     .flat_map(|(_, elems)| elems.iter_mut().filter(|f| to_snip.contains(f)))
            //     .for_each(|el| {
            //         let ty = funcs.get(*el).ty();
            //         *el = *unreachable_funcs
            //             .entry(ty)
            //             .or_insert_with(|| make_unreachable_func(ty, types, locals, funcs));
            //     });
        }
    }
}

// todo: we want to run our own gc pass
// /// Run GC passes over the module specified.
// pub fn run(m: &mut Module) {
//     use walrus::passes;
//     // let used = Used::new(m);

//     let mut unused_imports = Vec::new();
//     for import in m.imports.iter() {
//         let used = match &import.kind {
//             ImportKind::Function(f) => used.funcs.contains(f),
//             ImportKind::Table(t) => used.tables.contains(t),
//             ImportKind::Global(g) => used.globals.contains(g),
//             ImportKind::Memory(m) => used.memories.contains(m),
//         };
//         if !used {
//             unused_imports.push(import.id());
//         }
//     }
//     for id in unused_imports {
//         m.imports.delete(id);
//     }

//     for id in unused(&used.tables, m.tables.iter().map(|t| t.id())) {
//         m.tables.delete(id);
//     }
//     for id in unused(&used.globals, m.globals.iter().map(|t| t.id())) {
//         m.globals.delete(id);
//     }
//     for id in unused(&used.memories, m.memories.iter().map(|t| t.id())) {
//         m.memories.delete(id);
//     }
//     for id in unused(&used.data, m.data.iter().map(|t| t.id())) {
//         m.data.delete(id);
//     }
//     for id in unused(&used.elements, m.elements.iter().map(|t| t.id())) {
//         m.elements.delete(id);
//     }
//     for id in unused(&used.types, m.types.iter().map(|t| t.id())) {
//         m.types.delete(id);
//     }
//     for id in unused(&used.funcs, m.funcs.iter().map(|t| t.id())) {
//         m.funcs.delete(id);
//     }
// }

// // pub type IdHashSet<T> = HashSet<Id<T>, BuildIdHasher>;
// fn unused<T>(used: &IdHashSet<T>, all: impl Iterator<Item = Id<T>>) -> Vec<Id<T>> {
//     let mut unused = Vec::new();
//     for id in all {
//         if !used.contains(&id) {
//             unused.push(id);
//         }
//     }
//     unused
// }
