use anyhow::{anyhow, Result};
use jaq_core::load::{Arena, File, Loader};
use jaq_core::{data, unwrap_valr, Ctx, Vars};
use jaq_json::Val;

const TOON_WORLD_HELPERS: &str = r#"
def section($name): .sections[]? | select(.heading == $name);
def code($lang): .blocks[]? | select(.type == "code" and .lang == $lang);
def links: .links[]?;
"#;

pub fn execute(query: &str, input: Val) -> Result<Vec<Val>> {
    let defs = jaq_core::defs()
        .chain(jaq_std::defs())
        .chain(jaq_json::defs());
    let funs = jaq_core::funs()
        .chain(jaq_std::funs())
        .chain(jaq_json::funs());

    let source = format!("{TOON_WORLD_HELPERS}\n{query}");
    let loader = Loader::new(defs);
    let arena = Arena::default();
    let modules = loader
        .load(
            &arena,
            File {
                code: &source,
                path: (),
            },
        )
        .map_err(|errors| anyhow!("error[query]: parse failed: {errors:?}"))?;

    let filter = jaq_core::Compiler::default()
        .with_funs(funs)
        .compile(modules)
        .map_err(|errors| anyhow!("error[query]: compile failed: {errors:?}"))?;

    let ctx = Ctx::<data::JustLut<Val>>::new(&filter.lut, Vars::new([]));

    filter
        .id
        .run((ctx, input))
        .map(unwrap_valr)
        .map(|result| result.map_err(|error| anyhow!("error[query]: runtime failed: {error:?}")))
        .collect()
}
