//! Reading and validating `spec.yaml`, the single source of truth for an
//! indicator's metadata (`docs/SPEC_FORMAT.md`).
//!
//! Validation is strict on purpose. A spec that is wrong in a way the generator
//! tolerates becomes a function documented as one thing and computing another,
//! which is the failure this project is built to prevent, so an unknown key is
//! an error rather than a field quietly ignored.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::yaml::{Node, Reader, SpecError, parse};

pub const INPUT_KINDS: &[&str] = &[
    "series",
    "open",
    "high",
    "low",
    "close",
    "volume",
    "timestamps",
];
pub const DTYPES: &[&str] = &["float64", "int32"];
pub const PLOT_HINTS: &[&str] = &[
    "line",
    "histogram",
    "upper_band",
    "middle_band",
    "lower_band",
    "level",
    "direction",
    "pattern",
];
pub const FLAGS: &[&str] = &[
    "overlap",
    "path_dependent",
    "unstable",
    "requires_timestamps",
    "pattern",
    // The output can legitimately be NaN or infinite for finite input: a
    // logarithm of zero, an arc cosine outside [-1, 1], a division by zero.
    "nan_inf_output",
    // An int32 output is a row index into the input, so skipping leading
    // warm-up rows shifts it rather than leaving it unchanged.
    "absolute_index",
];

#[derive(Debug, Clone)]
/// Fields the reader validates but no emitter consumes yet: `optional` inputs,
/// `plot` hints and the TA-Lib output names all feed `registry.rs` and the
/// generated docs pages, the two outputs M2 still owes. They are validated now
/// so a spec written today does not have to be revisited then.
#[allow(dead_code)]
pub struct Input {
    pub name: String,
    pub kind: String,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamType {
    Int,
    Float,
    Bool,
    Tz,
    Enum(String),
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: ParamType,
    pub default: String,
    pub min: Option<String>,
    pub max: Option<String>,
    pub doc: String,
}

#[derive(Debug, Clone)]
/// Fields the reader validates but no emitter consumes yet: `optional` inputs,
/// `plot` hints and the TA-Lib output names all feed `registry.rs` and the
/// generated docs pages, the two outputs M2 still owes. They are validated now
/// so a spec written today does not have to be revisited then.
#[allow(dead_code)]
pub struct Output {
    pub name: String,
    pub dtype: String,
    pub plot: String,
    pub doc: String,
}

#[derive(Debug, Clone)]
/// Fields the reader validates but no emitter consumes yet: `optional` inputs,
/// `plot` hints and the TA-Lib output names all feed `registry.rs` and the
/// generated docs pages, the two outputs M2 still owes. They are validated now
/// so a spec written today does not have to be revisited then.
#[allow(dead_code)]
pub struct Talib {
    pub name: String,
    pub params: BTreeMap<String, String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone)]
/// `group`, `flags`, `references` and `see_also` are validated here and
/// consumed by `registry.rs` and the generated docs pages, the two outputs M2
/// still owes, so a spec written today does not have to be revisited then.
#[allow(dead_code)]
pub struct Spec {
    pub path: PathBuf,
    pub name: String,
    pub title: String,
    pub group: String,
    pub flags: Vec<String>,
    pub inputs: Vec<Input>,
    pub params: Vec<Param>,
    pub outputs: Vec<Output>,
    pub talib: Option<Talib>,
    pub references: Vec<String>,
    pub see_also: Vec<String>,
}

impl Spec {
    /// `sma` -> `Sma`, `cdl_doji` -> `CdlDoji`: the type the module must expose.
    pub fn type_name(&self) -> String {
        self.name
            .split('_')
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect()
    }

    pub fn stream_type(&self) -> String {
        format!("{}Stream", self.type_name())
    }

    /// The Python class name of the stream handle, e.g. `SmaStream`.
    pub fn py_stream_class(&self) -> String {
        self.stream_type()
    }
}

fn is_snake_case(value: &str) -> bool {
    !value.is_empty()
        && value.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn read_input(reader: &Reader<'_>) -> Result<Input, SpecError> {
    reader.only(&["name", "kind", "optional"])?;
    let name = reader.text("name")?;
    let kind = reader.text("kind")?;
    if !INPUT_KINDS.contains(&kind.as_str()) {
        return Err(reader.error(format!(
            "input `{name}` has kind `{kind}`; allowed: {}",
            INPUT_KINDS.join(", ")
        )));
    }
    if !is_snake_case(&name) {
        return Err(reader.error(format!("input name `{name}` is not lowercase snake_case")));
    }
    let optional = match reader.get("optional") {
        None => false,
        Some(flag) => flag.scalar()? == "true",
    };
    Ok(Input {
        name,
        kind,
        optional,
    })
}

fn read_param(
    reader: &Reader<'_>,
    enums: &BTreeMap<String, Vec<String>>,
) -> Result<Param, SpecError> {
    reader.only(&["name", "type", "default", "min", "max", "doc"])?;
    let name = reader.text("name")?;
    if !is_snake_case(&name) {
        return Err(reader.error(format!(
            "parameter name `{name}` is not lowercase snake_case"
        )));
    }
    let raw = reader.text("type")?;
    let ty = match raw.as_str() {
        "int" => ParamType::Int,
        "float" => ParamType::Float,
        "bool" => ParamType::Bool,
        "tz" => ParamType::Tz,
        other => match other.strip_prefix("enum:") {
            Some(enum_name) if enums.contains_key(enum_name) => {
                ParamType::Enum(enum_name.to_string())
            }
            Some(enum_name) => {
                return Err(reader.error(format!(
                    "parameter `{name}` uses enum `{enum_name}`, which _enums.yaml does not define"
                )));
            }
            None => {
                return Err(reader.error(format!(
                    "parameter `{name}` has type `{other}`; allowed: int, float, bool, tz, enum:<Name>"
                )));
            }
        },
    };

    let default = reader.text("default")?;
    let min = reader.optional_text("min")?;
    let max = reader.optional_text("max")?;
    let doc = reader.text("doc")?;

    match &ty {
        ParamType::Int => {
            let parsed: i64 = default.parse().map_err(|_| {
                reader.error(format!(
                    "parameter `{name}` default `{default}` is not an integer"
                ))
            })?;
            if let Some(low) = &min {
                let low: i64 = low.parse().map_err(|_| {
                    reader.error(format!("parameter `{name}` min `{low}` is not an integer"))
                })?;
                if parsed < low {
                    return Err(reader.error(format!(
                        "parameter `{name}` default {parsed} is below its min {low}"
                    )));
                }
            }
            if let Some(high) = &max {
                let high: i64 = high.parse().map_err(|_| {
                    reader.error(format!("parameter `{name}` max `{high}` is not an integer"))
                })?;
                if parsed > high {
                    return Err(reader.error(format!(
                        "parameter `{name}` default {parsed} is above its max {high}"
                    )));
                }
            }
        }
        ParamType::Float => {
            default.parse::<f64>().map_err(|_| {
                reader.error(format!(
                    "parameter `{name}` default `{default}` is not a number"
                ))
            })?;
        }
        ParamType::Bool => {
            if default != "true" && default != "false" {
                return Err(reader.error(format!(
                    "parameter `{name}` default `{default}` must be true or false"
                )));
            }
        }
        ParamType::Enum(enum_name) => {
            let values = &enums[enum_name];
            if !values.contains(&default) {
                return Err(reader.error(format!(
                    "parameter `{name}` default `{default}` is not a value of {enum_name}: {}",
                    values.join(", ")
                )));
            }
        }
        ParamType::Tz => {}
    }

    Ok(Param {
        name,
        ty,
        default,
        min,
        max,
        doc,
    })
}

fn read_output(reader: &Reader<'_>) -> Result<Output, SpecError> {
    reader.only(&["name", "dtype", "plot", "doc"])?;
    let name = reader.text("name")?;
    if !is_snake_case(&name) {
        return Err(reader.error(format!("output name `{name}` is not lowercase snake_case")));
    }
    let dtype = reader.text("dtype")?;
    if !DTYPES.contains(&dtype.as_str()) {
        return Err(reader.error(format!(
            "output `{name}` has dtype `{dtype}`; allowed: {}",
            DTYPES.join(", ")
        )));
    }
    let plot = reader.text("plot")?;
    if !PLOT_HINTS.contains(&plot.as_str()) {
        return Err(reader.error(format!(
            "output `{name}` has plot hint `{plot}`; allowed: {}",
            PLOT_HINTS.join(", ")
        )));
    }
    Ok(Output {
        name,
        dtype,
        plot,
        doc: reader.text("doc")?,
    })
}

pub fn read_spec(
    path: &Path,
    groups: &[String],
    enums: &BTreeMap<String, Vec<String>>,
) -> Result<Spec, SpecError> {
    let text = fs::read_to_string(path)
        .map_err(|error| SpecError::new(path, 1, format!("cannot read: {error}")))?;
    let node = parse(path, &text)?;
    let reader = Reader::new(path, &node);
    reader.only(&[
        "name",
        "title",
        "group",
        "flags",
        "inputs",
        "params",
        "outputs",
        "talib",
        "references",
        "see_also",
    ])?;

    let name = reader.text("name")?;
    let folder = path
        .parent()
        .and_then(Path::file_name)
        .and_then(|f| f.to_str())
        .unwrap_or_default();
    if name != folder {
        return Err(reader.error(format!(
            "`name: {name}` does not match the folder name `{folder}`"
        )));
    }
    if !is_snake_case(&name) {
        return Err(reader.error(format!("`name: {name}` is not lowercase snake_case")));
    }

    let title = reader.text("title")?;
    if title.ends_with('.') {
        return Err(reader.error("`title` must not end with a period"));
    }
    if title.contains('\n') {
        return Err(reader.error("`title` must be a single line"));
    }

    let group = reader.text("group")?;
    if !groups.contains(&group) {
        return Err(reader.error(format!(
            "`group: {group}` is not a key in _groups.yaml: {}",
            groups.join(", ")
        )));
    }

    let flags = reader.string_list("flags")?;
    for flag in &flags {
        if !FLAGS.contains(&flag.as_str()) {
            return Err(reader.error(format!(
                "unknown flag `{flag}`; allowed: {}",
                FLAGS.join(", ")
            )));
        }
    }

    let inputs: Vec<Input> = reader
        .list("inputs")?
        .iter()
        .map(read_input)
        .collect::<Result<_, _>>()?;
    if inputs.is_empty() {
        return Err(reader.error("`inputs` must list at least one input"));
    }

    let params: Vec<Param> = reader
        .list("params")?
        .iter()
        .map(|item| read_param(item, enums))
        .collect::<Result<_, _>>()?;

    let outputs: Vec<Output> = reader
        .list("outputs")?
        .iter()
        .map(read_output)
        .collect::<Result<_, _>>()?;
    if outputs.is_empty() {
        return Err(reader.error("`outputs` must list at least one output"));
    }

    let input_names: Vec<&str> = inputs.iter().map(|i| i.name.as_str()).collect();
    let mut seen: Vec<&str> = Vec::new();
    for output in &outputs {
        if input_names.contains(&output.name.as_str()) {
            return Err(reader.error(format!("output `{}` repeats an input name", output.name)));
        }
        if seen.contains(&output.name.as_str()) {
            return Err(reader.error(format!("output `{}` is listed twice", output.name)));
        }
        seen.push(&output.name);
    }

    let talib = match reader.get("talib") {
        None => None,
        Some(block) => {
            block.only(&["name", "params", "outputs"])?;
            let alias = block.text("name")?;
            let mut renames = BTreeMap::new();
            if let Some(map) = block.get("params") {
                for (key, value) in map.pairs()? {
                    renames.insert(key, value.scalar()?.to_string());
                }
            }
            for ours in renames.keys() {
                if !params.iter().any(|p| &p.name == ours) {
                    return Err(block.error(format!(
                        "talib params maps `{ours}`, which is not a parameter of this indicator"
                    )));
                }
            }
            let alias_outputs = block.string_list("outputs")?;
            if alias_outputs.len() != outputs.len() {
                return Err(block.error(format!(
                    "talib lists {} outputs but the indicator has {}",
                    alias_outputs.len(),
                    outputs.len()
                )));
            }
            Some(Talib {
                name: alias,
                params: renames,
                outputs: alias_outputs,
            })
        }
    };

    Ok(Spec {
        path: path.to_path_buf(),
        name,
        title,
        group,
        flags,
        inputs,
        params,
        outputs,
        talib,
        references: reader.string_list("references")?,
        see_also: reader.string_list("see_also")?,
    })
}

pub fn read_groups(path: &Path) -> Result<Vec<(String, String)>, SpecError> {
    let text = fs::read_to_string(path)
        .map_err(|error| SpecError::new(path, 1, format!("cannot read: {error}")))?;
    let node = parse(path, &text)?;
    let reader = Reader::new(path, &node);
    let Node::List(items, _) = &node else {
        return Err(reader.error("_groups.yaml must be a list of { key, title }"));
    };
    items
        .iter()
        .map(|item| {
            let entry = Reader::new(path, item);
            entry.only(&["key", "title"])?;
            Ok((entry.text("key")?, entry.text("title")?))
        })
        .collect()
}

pub fn read_enums(path: &Path) -> Result<BTreeMap<String, Vec<String>>, SpecError> {
    let text = fs::read_to_string(path)
        .map_err(|error| SpecError::new(path, 1, format!("cannot read: {error}")))?;
    let node = parse(path, &text)?;
    let reader = Reader::new(path, &node);
    let mut found = BTreeMap::new();
    for (name, body) in reader.pairs()? {
        body.only(&["values", "talib_int"])?;
        found.insert(name, body.string_list("values")?);
    }
    Ok(found)
}

/// Every spec in the tree, sorted by name, with catalogue-wide checks applied.
pub fn read_all(indicators: &Path) -> Result<Vec<Spec>, Vec<SpecError>> {
    let groups = match read_groups(&indicators.join("_groups.yaml")) {
        Ok(groups) => groups,
        Err(error) => return Err(vec![error]),
    };
    let enums = match read_enums(&indicators.join("_enums.yaml")) {
        Ok(enums) => enums,
        Err(error) => return Err(vec![error]),
    };
    let keys: Vec<String> = groups.iter().map(|(key, _)| key.clone()).collect();

    let mut folders: Vec<PathBuf> = match fs::read_dir(indicators) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.join("spec.yaml").is_file())
            .collect(),
        Err(error) => {
            return Err(vec![SpecError::new(
                indicators,
                1,
                format!("cannot list: {error}"),
            )]);
        }
    };
    folders.sort();

    let mut specs = Vec::new();
    let mut errors = Vec::new();
    for folder in folders {
        match read_spec(&folder.join("spec.yaml"), &keys, &enums) {
            Ok(spec) => specs.push(spec),
            Err(error) => errors.push(error),
        }
    }

    // Catalogue-wide uniqueness: two indicators must not claim the same
    // function name or the same output name, or a DataFrame join would collide.
    let mut names: BTreeMap<String, String> = BTreeMap::new();
    for spec in &specs {
        if let Some(other) = names.insert(spec.name.clone(), spec.name.clone()) {
            errors.push(SpecError::new(
                &spec.path,
                1,
                format!("function name `{other}` is used by another indicator"),
            ));
        }
        for output in &spec.outputs {
            if let Some(other) = names.insert(output.name.clone(), spec.name.clone())
                && other != spec.name
            {
                errors.push(SpecError::new(
                    &spec.path,
                    1,
                    format!("output `{}` is already used by `{other}`", output.name),
                ));
            }
        }
    }

    specs.sort_by(|a, b| a.name.cmp(&b.name));
    if errors.is_empty() {
        Ok(specs)
    } else {
        Err(errors)
    }
}
