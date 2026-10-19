//! Local experiment: verify slim capability boundaries, not production tests.
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::layout::PagedDocument;
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
struct ProbeWorld { library: LazyHash<Library>, book: LazyHash<FontBook>, fonts: Vec<Font>, source: Source }
impl ProbeWorld {
 fn new(body: &str) -> Self {
  let fonts: Vec<_> = [
   include_bytes!("../../fonts/DejaVuSans.ttf").as_slice(),
   include_bytes!("../../fonts/NewCMMath-Regular.otf").as_slice(),
  ].into_iter().flat_map(|b| Font::iter(Bytes::new(b.to_vec()))).collect();
  Self { library: LazyHash::new(Library::default()), book: LazyHash::new(FontBook::from_fonts(&fonts)), fonts,
   source: Source::new(FileId::new_fake(VirtualPath::new("/main.typ")), format!("#set page(width: auto, height: auto, margin: 4pt)\n#set text(font: \"DejaVu Sans\")\n{body}")) }
 }
}
impl World for ProbeWorld {
 fn library(&self) -> &LazyHash<Library> { &self.library }
 fn book(&self) -> &LazyHash<FontBook> { &self.book }
 fn main(&self) -> FileId { self.source.id() }
 fn source(&self, id: FileId) -> FileResult<Source> { if id==self.main() { Ok(self.source.clone()) } else { Err(FileError::NotFound(id.vpath().as_rooted_path().into())) } }
 fn file(&self, id: FileId) -> FileResult<Bytes> { Err(FileError::NotFound(id.vpath().as_rooted_path().into())) }
 fn font(&self, index: usize) -> Option<Font> { self.fonts.get(index).cloned() }
 fn today(&self, _: Option<i64>) -> Option<Datetime> { Datetime::from_ymd(2026,10,18) }
}
fn main() {
 let slim = std::env::args().nth(1).as_deref()==Some("slim");
 let cases = [
  ("math", "Ramsey $T_2^*$ and $frac(1, sqrt(2))$", None),
  ("raw text", "```rust\nfn main() {}\n```", None),
  ("ordinary reference", "#set heading(numbering: \"1.\")\n= Intro <intro>\nSee @intro.", None),
  ("plugin", "#plugin(bytes(\"invalid wasm\"))", Some("WebAssembly plugins are disabled")),
  ("valid plugin", "#let p = plugin(bytes((0,97,115,109,1,0,0,0,5,3,1,0,1,7,10,1,6,109,101,109,111,114,121,2,0)))", Some("WebAssembly plugins are disabled")),
  ("bibliography", "#bibliography(bytes(\"@book{book, title={A book}, author={Doe, Jane}, year={2020}}\"))", Some("bibliography support is disabled")),
  ("citation", "#cite(<book>)", Some("bibliography support is disabled")),
  ("citation style", "#cite(<book>, style: \"apa\")", Some("bibliography support is disabled")),
  ("custom highlighting", "#raw(\"fn main() {}\", syntaxes: \"custom.sublime-syntax\")", Some("syntax highlighting is disabled")),
 ];
 for (name, body, disabled) in cases {
  let result=typst::compile::<PagedDocument>(&ProbeWorld::new(body)).output;
  let messages=result.as_ref().err().map(|e| e.iter().map(|d|d.message.to_string()).collect::<Vec<_>>().join("; ")).unwrap_or_default();
  if disabled.is_none() { assert!(result.is_ok(), "{name}: {messages}"); }
  else if slim { assert!(messages.contains(disabled.unwrap()), "{name}: {messages}"); }
  else { assert!(!messages.contains("disabled"), "{name}: {messages}");
   if name=="valid plugin" || name=="bibliography" { assert!(result.is_ok(), "{name}: {messages}"); }
  }
  println!("PASS {name}: {}", if result.is_ok() { "compiled" } else { &messages });
 }
}
