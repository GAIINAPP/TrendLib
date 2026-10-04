//! A YAML tree that remembers which line each node came from.
//!
//! `yaml-rust2` hands back values without positions, but a spec validator whose
//! errors cannot be clicked is half a validator, so the event stream is
//! assembled here instead: every event arrives with a marker, and the marker is
//! kept alongside the value.

use std::fmt;
use std::path::{Path, PathBuf};

use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser};
use yaml_rust2::scanner::Marker;

#[derive(Debug, Clone)]
pub enum Node {
    Scalar(String, usize),
    List(Vec<Node>, usize),
    Map(Vec<(String, Node)>, usize),
}

impl Node {
    pub fn line(&self) -> usize {
        match self {
            Self::Scalar(_, line) | Self::List(_, line) | Self::Map(_, line) => *line,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Scalar(..) => "a value",
            Self::List(..) => "a list",
            Self::Map(..) => "a map",
        }
    }
}

/// An error that names the file, the line and the key it is about.
#[derive(Debug, Clone)]
pub struct SpecError {
    pub path: PathBuf,
    pub line: usize,
    pub message: String,
}

impl SpecError {
    pub fn new(path: &Path, line: usize, message: impl Into<String>) -> Self {
        Self {
            path: path.to_path_buf(),
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for SpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.path.display(), self.line, self.message)
    }
}

enum Partial {
    List(Vec<Node>, usize),
    Map(Vec<(String, Node)>, usize, Option<String>),
}

#[derive(Default)]
struct Builder {
    stack: Vec<Partial>,
    root: Option<Node>,
    failure: Option<String>,
}

impl Builder {
    fn place(&mut self, node: Node) {
        match self.stack.last_mut() {
            None => self.root = Some(node),
            Some(Partial::List(items, _)) => items.push(node),
            Some(Partial::Map(entries, _, pending)) => match pending.take() {
                Some(key) => entries.push((key, node)),
                None => match node {
                    Node::Scalar(key, _) => *pending = Some(key),
                    other => {
                        self.failure
                            .get_or_insert_with(|| format!("{} cannot be a key", other.kind()));
                    }
                },
            },
        }
    }
}

impl MarkedEventReceiver for Builder {
    fn on_event(&mut self, event: Event, mark: Marker) {
        let line = mark.line();
        match event {
            Event::Scalar(value, ..) => self.place(Node::Scalar(value, line)),
            Event::SequenceStart(..) => self.stack.push(Partial::List(Vec::new(), line)),
            Event::MappingStart(..) => self.stack.push(Partial::Map(Vec::new(), line, None)),
            Event::SequenceEnd => {
                if let Some(Partial::List(items, at)) = self.stack.pop() {
                    self.place(Node::List(items, at));
                }
            }
            Event::MappingEnd => {
                if let Some(Partial::Map(entries, at, _)) = self.stack.pop() {
                    self.place(Node::Map(entries, at));
                }
            }
            Event::Alias(_) => {
                self.failure
                    .get_or_insert_with(|| "anchors and aliases are not allowed in a spec".into());
            }
            _ => {}
        }
    }
}

pub fn parse(path: &Path, text: &str) -> Result<Node, SpecError> {
    let mut builder = Builder::default();
    Parser::new_from_str(text)
        .load(&mut builder, false)
        .map_err(|error| SpecError::new(path, error.marker().line(), error.to_string()))?;
    if let Some(failure) = builder.failure {
        return Err(SpecError::new(path, 1, failure));
    }
    builder
        .root
        .ok_or_else(|| SpecError::new(path, 1, "the file is empty"))
}

/// A cursor over a node that reports what it could not find, and where.
pub struct Reader<'a> {
    pub path: &'a Path,
    node: &'a Node,
}

impl<'a> Reader<'a> {
    pub fn new(path: &'a Path, node: &'a Node) -> Self {
        Self { path, node }
    }

    pub fn error(&self, message: impl Into<String>) -> SpecError {
        SpecError::new(self.path, self.node.line(), message)
    }

    pub fn at(&self, line: usize, message: impl Into<String>) -> SpecError {
        SpecError::new(self.path, line, message)
    }

    fn entries(&self) -> Result<&'a [(String, Node)], SpecError> {
        match self.node {
            Node::Map(entries, _) => Ok(entries),
            other => Err(self.error(format!("expected a map, found {}", other.kind()))),
        }
    }

    /// Reject any key the format does not define, so a typo is an error rather
    /// than a silently ignored field.
    pub fn only(&self, allowed: &[&str]) -> Result<(), SpecError> {
        for (key, value) in self.entries()? {
            if !allowed.contains(&key.as_str()) {
                return Err(self.at(
                    value.line(),
                    format!("unknown key `{key}`; allowed here: {}", allowed.join(", ")),
                ));
            }
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<Reader<'a>> {
        self.entries()
            .ok()?
            .iter()
            .find_map(|(found, node)| (found == key).then(|| Reader::new(self.path, node)))
    }

    pub fn required(&self, key: &str) -> Result<Reader<'a>, SpecError> {
        self.get(key)
            .ok_or_else(|| self.error(format!("missing required key `{key}`")))
    }

    pub fn scalar(&self) -> Result<&'a str, SpecError> {
        match self.node {
            Node::Scalar(value, _) => Ok(value),
            other => Err(self.error(format!("expected a value, found {}", other.kind()))),
        }
    }

    pub fn text(&self, key: &str) -> Result<String, SpecError> {
        let reader = self.required(key)?;
        let value = reader.scalar()?;
        if value.trim() != value {
            return Err(reader.error(format!("`{key}` has leading or trailing whitespace")));
        }
        if value.is_empty() {
            return Err(reader.error(format!("`{key}` is empty")));
        }
        Ok(value.to_string())
    }

    pub fn optional_text(&self, key: &str) -> Result<Option<String>, SpecError> {
        match self.get(key) {
            None => Ok(None),
            Some(_) => self.text(key).map(Some),
        }
    }

    pub fn list(&self, key: &str) -> Result<Vec<Reader<'a>>, SpecError> {
        match self.get(key) {
            None => Ok(Vec::new()),
            Some(reader) => match reader.node {
                Node::List(items, _) => {
                    Ok(items.iter().map(|n| Reader::new(self.path, n)).collect())
                }
                other => {
                    Err(reader.error(format!("`{key}` must be a list, found {}", other.kind())))
                }
            },
        }
    }

    pub fn string_list(&self, key: &str) -> Result<Vec<String>, SpecError> {
        self.list(key)?
            .iter()
            .map(|item| item.scalar().map(str::to_string))
            .collect()
    }

    pub fn pairs(&self) -> Result<Vec<(String, Reader<'a>)>, SpecError> {
        Ok(self
            .entries()?
            .iter()
            .map(|(key, node)| (key.clone(), Reader::new(self.path, node)))
            .collect())
    }
}
