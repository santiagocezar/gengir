use if_chain::if_chain;

use crate::declarations::{Function, Namespace, Type};

macro_rules! typ {
    (any) => {
        Type::Any
    };
    ($prim:ident) => {
        Type::Primitive(String::from(stringify!($prim)))
    };
    ($prim:expr) => {
        Type::Primitive(String::from($prim))
    };
    (.$cls:ident) => {
        Type::LocalClass(String::from(stringify!($prim)))
    };
    ($mod:ident.$cls:ident) => {
        Type::ExternalClass {
            module: String::from(stringify!($mod)),
            name: String::from(stringify!($cls)),
        }
    };
}

/// This applies overrides based on the
/// [`gi.overrides`](https://gitlab.gnome.org/GNOME/pygobject/-/tree/master/gi/overrides)
/// module in PyGObject and some trial and error.
pub fn apply_overrides(ns: &mut Namespace) {
    let name = ns.name.clone();
    let mut t = Transofrmer(ns);

    match name.as_str() {
        "Gio" => t.transform_method("Application", "run", |run| {
            run.clear_parameters().add_self_param().add_named_param(
                "argv",
                typ!("list[str]"),
                true,
                "The process command line arguments. Uses `sys.argv` if None",
            )
        }),
        "Gtk" => t.transform_method("Button", "__init__", |run| {
            run.clear_parameters()
                .add_self_param()
                .add_star_param()
                .add_named_param(
                    "label",
                    typ!(str),
                    false,
                    "Text displayed inside the button",
                )
                .add_named_param("use_stock", typ!(bool), false, None)
                .add_named_param("use_underline", typ!(bool), false, None)
        }),
        _ => (),
    }
}

struct Transofrmer<'a>(&'a mut Namespace);

impl<'a> Transofrmer<'a> {
    fn transform_method(
        &mut self,
        class: &str,
        method: &str,
        tfn: impl FnOnce(Function) -> Function,
    ) {
        let classes = &mut self.0.classes; // alias

        if_chain! {
            // get the index where the class is placed
            if let Some(origin) = classes.get_index_of(class);
            // swapping is faster than shifting
            if let Some(mut class_decl) = classes.swap_take(class);
            if let Some(method_decl) = class_decl.methods.swap_take(method);
            then {
                class_decl.methods.insert(tfn(method_decl));

                // insert the class
                classes.insert(class_decl);

                // and put it back in place, maintaining the topological order
                classes.swap_indices(classes.len() - 1, origin)
            } else {
                return
            }
        }

        // self.0
        //     .classes
        //     .get(class)
        //     .map(|c| c.methods.get(method))
        //     .flatten()
        //     .map(|f| tfn(f))
    }
}
