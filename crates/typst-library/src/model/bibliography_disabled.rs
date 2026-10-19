//! Bibliography API shells for builds without the bibliography feature.
//!
//! Keep the elements and signatures used by evaluation, realization, and the
//! exporters available, while avoiding a dependency on the bibliography engine.
//! Actual bibliography/style loading and citation generation return diagnostics.

use std::num::NonZeroUsize;
use std::sync::Arc;

use comemo::Tracked;
use ecow::EcoString;
use rustc_hash::FxHashMap;
use typst_syntax::Spanned;
use typst_utils::NonZeroExt;

use crate::World;
use crate::diag::{
    HintedStrResult, LoadError, LoadResult, ReportPos, SourceResult, StrResult, bail,
};
use crate::engine::Engine;
use crate::foundations::{
    Bytes, CastInfo, Content, Derived, FromValue, IntoValue, Label, OneOrMultiple,
    Packed, Reflect, ShowSet, Smart, StyleChain, Styles, Synthesize, Value, elem,
};
use crate::introspection::{Introspector, Locatable, Location};
use crate::layout::{Em, PadElem};
use crate::loading::DataSource;
use crate::model::HeadingElem;
use crate::text::{Lang, LocalName, Region, TextElem};

/// A bibliography / reference listing.
///
/// This element requires the `bibliography` Cargo feature. Its type remains
/// available so document elements and exporters can share their usual APIs.
#[elem(Locatable, Synthesize, ShowSet, LocalName)]
pub struct BibliographyElem {
    /// One or multiple bibliography file paths or raw byte sources.
    #[required]
    #[parse(
        let sources = args.expect("sources")?;
        Bibliography::load(engine.world, sources)?
    )]
    pub sources: Derived<OneOrMultiple<DataSource>, Bibliography>,

    /// The title of the bibliography.
    pub title: Smart<Option<Content>>,

    /// Whether to include works that were not cited in the document.
    #[default(false)]
    pub full: bool,

    /// The bibliography style.
    #[parse(match args.named::<Spanned<CslSource>>("style")? {
        Some(source) => Some(CslStyle::load(engine, source)?),
        None => None,
    })]
    #[default(Derived::new(
        CslSource::Normal(DataSource::Bytes(Bytes::new([]))),
        CslStyle(()),
    ))]
    pub style: Derived<CslSource, CslStyle>,

    /// The language setting where the bibliography is.
    #[internal]
    #[synthesized]
    pub lang: Lang,

    /// The region setting where the bibliography is.
    #[internal]
    #[synthesized]
    pub region: Option<Region>,
}

impl BibliographyElem {
    /// Find the document's bibliography.
    pub fn find(_: Tracked<Introspector>) -> StrResult<Packed<Self>> {
        bail!("bibliography support is disabled; enable the `bibliography` Cargo feature")
    }

    /// Whether the bibliography contains the given key.
    ///
    /// Returning false lets ordinary document references continue to resolve.
    pub fn has(_: &Engine, _: Label) -> bool {
        false
    }

    /// Find all bibliography keys.
    pub fn keys(_: Tracked<Introspector>) -> Vec<(Label, Option<EcoString>)> {
        vec![]
    }
}

impl Packed<BibliographyElem> {
    /// Produces the heading for the bibliography, if any.
    pub fn realize_title(&self, styles: StyleChain) -> Option<Content> {
        self.title
            .get_cloned(styles)
            .unwrap_or_else(|| {
                Some(TextElem::packed(Packed::<BibliographyElem>::local_name_in(styles)))
            })
            .map(|title| {
                HeadingElem::new(title)
                    .with_depth(NonZeroUsize::ONE)
                    .pack()
                    .spanned(self.span())
            })
    }
}

impl Synthesize for Packed<BibliographyElem> {
    fn synthesize(&mut self, _: &mut Engine, _: StyleChain) -> SourceResult<()> {
        bail!(
            self.span(),
            "bibliography support is disabled; enable the `bibliography` Cargo feature"
        )
    }
}

impl ShowSet for Packed<BibliographyElem> {
    fn show_set(&self, _: StyleChain) -> Styles {
        const INDENT: Em = Em::new(1.0);
        let mut out = Styles::new();
        out.set(HeadingElem::numbering, None);
        out.set(PadElem::left, INDENT.into());
        out
    }
}

impl LocalName for Packed<BibliographyElem> {
    const KEY: &'static str = "bibliography";
}

/// A loaded bibliography, unavailable without the bibliography feature.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Bibliography(());

impl Bibliography {
    fn load(
        _: Tracked<dyn World + '_>,
        sources: Spanned<OneOrMultiple<DataSource>>,
    ) -> SourceResult<Derived<OneOrMultiple<DataSource>, Self>> {
        bail!(
            sources.span,
            "bibliography support is disabled; enable the `bibliography` Cargo feature"
        )
    }
}

/// A loaded CSL style, unavailable without the bibliography feature.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct CslStyle(());

impl CslStyle {
    /// Load a CSL style from a data source.
    pub fn load(
        _: &mut Engine,
        source: Spanned<CslSource>,
    ) -> SourceResult<Derived<CslSource, Self>> {
        bail!(
            source.span,
            "bibliography support is disabled; enable the `bibliography` Cargo feature"
        )
    }

    /// Load a CSL style from file contents.
    pub fn from_data(_: &Bytes) -> LoadResult<CslStyle> {
        Err(LoadError::new(
            ReportPos::None,
            "bibliography support is disabled",
            "enable the `bibliography` Cargo feature",
        ))
    }
}

/// Source for a CSL style.
///
/// Built-in style names are resolved only when bibliography support is enabled.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum CslSource {
    /// A data source, which will report the disabled feature when loaded.
    Normal(DataSource),
}

impl Reflect for CslSource {
    fn input() -> CastInfo {
        DataSource::input()
    }

    fn output() -> CastInfo {
        DataSource::output()
    }

    fn castable(value: &Value) -> bool {
        DataSource::castable(value)
    }
}

impl FromValue for CslSource {
    fn from_value(value: Value) -> HintedStrResult<Self> {
        DataSource::from_value(value).map(Self::Normal)
    }
}

impl IntoValue for CslSource {
    fn into_value(self) -> Value {
        match self {
            Self::Normal(source) => source.into_value(),
        }
    }
}

/// Fully formatted citations and references.
pub struct Works {
    /// Maps from the location of a citation group to its rendered content.
    pub citations: FxHashMap<Location, SourceResult<Content>>,
    /// Lists all references in the bibliography, with optional prefix.
    pub references: Option<Vec<(Option<Content>, Content, Location)>>,
    /// Whether the bibliography should have hanging indent.
    pub hanging_indent: bool,
}

impl Works {
    /// Generate all citations and the whole bibliography.
    pub fn generate(_: &Engine) -> StrResult<Arc<Works>> {
        bail!("bibliography support is disabled; enable the `bibliography` Cargo feature")
    }

    /// Extract the generated references.
    pub fn references<'a>(
        &'a self,
        elem: &Packed<BibliographyElem>,
        _: StyleChain,
    ) -> SourceResult<&'a [(Option<Content>, Content, Location)]> {
        bail!(
            elem.span(),
            "bibliography support is disabled; enable the `bibliography` Cargo feature"
        )
    }
}

/// Translation of `font-weight="light"` in CSL.
#[elem]
pub struct CslLightElem {
    #[required]
    pub body: Content,
}

/// Translation of `display="indent"` in CSL.
#[elem]
pub struct CslIndentElem {
    #[required]
    pub body: Content,
}

use crate::foundations::NativeElement;
