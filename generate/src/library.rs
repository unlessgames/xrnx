use crate::{error::Error, json::JsonDoc, sources::Source, types::*};
use std::{collections::HashMap, path::PathBuf};

#[derive(Clone)]
pub struct Library {
    // pub defs: Vec<Def>,
    pub classes: HashMap<String, Class>,
    pub enums: HashMap<String, Enum>,
    pub aliases: HashMap<String, Alias>,
}

impl Library {
    pub fn from_path(path: PathBuf) -> Result<Self, Error> {
        let mut defs: Vec<Def> = vec![];
        let sources = Source::from_path(path)?;
        if let Some(sources) = sources {
            for path in sources.file_paths(vec![]) {
                let definitions = JsonDoc::get(&path)?;
                defs.append(
                    &mut definitions
                        .iter()
                        .filter_map(Def::from_definition)
                        .collect::<Vec<Def>>(),
                );
                println!("{:?}", path)
            }
        }
        Ok(Self::from_defs(defs))
    }

    fn resolve_string(&self, s: String) -> Option<Kind> {
        if self.classes.contains_key(&s) {
            Some(Kind::Class(s))
        } else if self.aliases.contains_key(&s) {
            Some(Kind::Alias(s))
        } else if self.enums.contains_key(&s) {
            Some(Kind::EnumRef(s))
        } else {
            None
        }
    }

    fn resolve_kind(&self, kind: &Kind) -> Kind {
        match kind.clone() {
            Kind::Unresolved(s) => self.resolve_string(s).unwrap_or(kind.clone()),
            Kind::Array(bk) => Kind::Array(Box::new(self.resolve_kind(bk.as_ref()))),
            Kind::Nullable(bk) => Kind::Nullable(Box::new(self.resolve_kind(bk.as_ref()))),
            Kind::Table(key, value) => Kind::Table(
                Box::new(self.resolve_kind(key.as_ref())),
                Box::new(self.resolve_kind(value.as_ref())),
            ),
            Kind::Enum(kinds) => Kind::Enum(kinds.iter().map(|k| self.resolve_kind(k)).collect()),
            Kind::Function(f) => {
                let mut fun = f.clone();
                self.resolve_function(&mut fun);
                Kind::Function(fun)
            }
            Kind::Variadic(v) => Kind::Variadic(Box::new(self.resolve_kind(v.as_ref()))),
            Kind::Object(hm) => {
                let mut obj = hm.clone();
                for (key, value) in hm.iter() {
                    obj.insert(key.clone(), Box::new(self.resolve_kind(value.as_ref())));
                }
                Kind::Object(obj)
            }
            _ => kind.clone(),
        }
    }

    fn resolve_function(&self, f: &mut Function) {
        for p in f.params.iter_mut() {
            p.kind = self.resolve_kind(&p.kind)
        }
        for r in f.returns.iter_mut() {
            r.kind = self.resolve_kind(&r.kind)
        }
    }

    fn resolve_classes(&mut self) {
        let l = self.clone();
        for (_, c) in self.classes.iter_mut() {
            for f in c.fields.iter_mut() {
                f.kind = l.resolve_kind(&f.kind)
            }
            for f in c.methods.iter_mut() {
                l.resolve_function(f)
            }
        }
    }

    fn class_desc(name: &str, desc: &str) -> Class {
        Class {
            name: name.to_string(),
            desc: desc.to_string(),
            fields: vec![],
            methods: vec![],
            enums: vec![],
        }
    }

    fn builtin_classes() -> Vec<Class> {
        vec![
            Self::class_desc(
                "nil",
                "A built-in type representing a non-existant value, [see details](https://www.lua.org/pil/2.1.html). When you see `?` at the end of types, it means they can be nil.",
            ),
            Self::class_desc(
                "boolean",
                "A built-in type representing a boolean (true or false) value, [see details](https://www.lua.org/pil/2.2.html)",
            ),
            Self::class_desc(
                "number",
                "A built-in type representing floating point numbers, [see details](https://www.lua.org/pil/2.3.html)",
            ),
            Self::class_desc(
                "string",
                "A built-in type representing a string of characters, [see details](https://www.lua.org/pil/2.4.html)",
            ),
            Self::class_desc("function", "A built-in type representing functions, [see details](https://www.lua.org/pil/2.6.html)"),
            Self::class_desc("table", "A built-in type representing associative arrays, [see details](https://www.lua.org/pil/2.5.html)"),
            Self::class_desc("userdata", "A built-in type representing array values, [see details](https://www.lua.org/pil/28.1.html)."),
            Self::class_desc(
                "lightuserdata",
                "A built-in type representing a pointer, [see details](https://www.lua.org/pil/28.5.html)",
            ),

            Self::class_desc("integer", "A helper type that represents whole numbers, a subset of [number](number.md)"),
            Self::class_desc(
                "any",
                "A type for a dynamic argument, it can be anything at run-time.",
            ),
            Self::class_desc(
                "unknown",
                "A dummy type for something that cannot be inferred before run-time.",
            ),
            Self::class_desc(
                "self",
                "A type that represents an instance that you call a function on. When you see a function signature starting with this type, you should use `:` to call the function on the instance, this way you can omit this first argument.\n\
                ```lua
                local song = renoise.song()
                local first = song:pattern(0)
                ```",
            ),
        ]
    }

    fn from_defs(defs: Vec<Def>) -> Self {
        let mut classes = HashMap::new();
        let mut enums = HashMap::new();
        let mut aliases = HashMap::new();
        let mut dangling_functions = vec![];
        for d in defs.iter() {
            match d {
                Def::Alias(a) => {
                    aliases.insert(a.name.clone(), a.clone());
                }
                Def::Enum(e) => {
                    enums.insert(e.name.clone(), e.clone());
                }
                Def::Class(c) => {
                    classes.insert(c.name.clone(), c.clone());
                }
                Def::Function(f) => dangling_functions.push(f.clone()),
            }
        }

        let mut l = Self {
            // defs,
            classes,
            enums,
            aliases,
        };

        // transform Kind::Unresolved(xy) to the approriate classes and aliases
        // by cross referencing the hash maps on the library
        l.resolve_classes();
        let mut aliases = l.aliases.clone();
        aliases
            .iter_mut()
            .for_each(|(_, a)| a.kind = l.resolve_kind(&a.kind));
        l.aliases = aliases;
        dangling_functions
            .iter_mut()
            .for_each(|f| l.resolve_function(f));

        // add enums to their respective classes
        for (k, e) in l.enums.iter() {
            let base = Class::get_base(k);
            if let Some(base) = base {
                if let Some(class) = l.classes.get_mut(base) {
                    class.enums.push(e.clone())
                }
            }
        }

        let builtin_classes = Self::builtin_classes();
        builtin_classes.iter().for_each(|c| {
            l.classes.insert(c.name.clone(), c.clone());
        });

        // create dummy classes for global functions and those without a parent class (like bit)
        for f in dangling_functions.iter_mut() {
            let name = &f.name.clone().unwrap_or_default();
            let base = Class::get_base(name).unwrap_or("global");
            if let Some((_, class)) = l.classes.iter_mut().find(|(k, _)| *k == base) {
                if class.methods.iter().any(|fun| fun.name == f.name) {
                    // skip class function as it was already added by Def::from_definition to its class
                } else {
                    class.methods.push(f.clone())
                }
            } else {
                l.classes.insert(
                    base.to_string(),
                    Class {
                        name: base.to_string(),
                        methods: vec![f.clone()],
                        fields: vec![],
                        enums: vec![],
                        // TODO the description should somehow end up here from bit, os etc
                        desc: String::default(),
                    },
                );
            }
        }

        for c in l.classes.values() {
            println!("{}", c.show());
        }
        println!("classes");
        for k in l.classes.keys() {
            println!("  {}", k);
            if !builtin_classes.iter().any(|c| c.name.as_str() == k)
                && l.classes.get(k).unwrap().is_empty()
            {
                println!("  \x1b[33m^--- has no fields, methods or enums\x1b[0m")
            }
        }
        println!("enums");
        for k in l.enums.keys() {
            println!("  {}", k);
        }

        // println!("aliases");
        // for a in l.aliases.values() {
        //     println!("  {}", a.show());
        // }

        l
    }
}
