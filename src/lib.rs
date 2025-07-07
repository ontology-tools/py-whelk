use horned_owl::model::{
    AnnotatedComponent, ArcAnnotatedComponent, ArcStr, ClassExpression, Component, SubClassOf,
};
use horned_owl::ontology::indexed::OntologyIndex;
use horned_owl::ontology::set::SetOntology;
use horned_owl::{model as ho, vocab};

use pyhornedowlreasoner::{PyReasoner, Reasoner, ReasonerError, export_py_reasoner};
use whelk::whelk::model::Axiom;
use whelk::whelk::owl::{translate_axiom, translate_ontology};
use whelk::whelk::reasoner::{ReasonerState, assert, assert_append};

pub struct PyWhelkReasoner {
    state: ReasonerState,
    pending_insert: Vec<ArcAnnotatedComponent>,
}

export_py_reasoner!(PyWhelkReasoner);

impl PyReasoner for PyWhelkReasoner {
    fn create_reasoner(ontology: SetOntology<ArcStr>) -> Self {
        let translated = translate_ontology(&ontology);

        PyWhelkReasoner {
            state: assert(&translated),
            pending_insert: Vec::new(),
        }
    }
}

impl OntologyIndex<ArcStr, ArcAnnotatedComponent> for PyWhelkReasoner {
    fn index_insert(&mut self, cmp: ArcAnnotatedComponent) -> bool {
        self.pending_insert.push(cmp);

        false
    }

    fn index_remove(&mut self, _cmp: &AnnotatedComponent<ArcStr>) -> bool {
        false
    }
}

impl Reasoner<ArcStr, ArcAnnotatedComponent> for PyWhelkReasoner {
    fn get_name(&self) -> String {
        "PyWhelk".to_string()
    }

    fn flush(&mut self) -> Result<(), ReasonerError> {
        let pending_inserts = std::mem::take(&mut self.pending_insert);

        let translated = pending_inserts
            .into_iter()
            .flat_map(|c| translate_axiom(&c.component))
            .filter_map(|c| match c.as_ref() {
                Axiom::ConceptInclusion(ci) => Some(ci.clone()),
                _ => None,
            })
            .collect();
        self.state = assert_append(&translated, &self.state);

        Ok(())
    }

    fn inferred_axioms(&self) -> Box<dyn Iterator<Item = Component<ArcStr>>> {
        let build = ho::Build::<ArcStr>::new();

        Box::new(
            self.state
                .named_subsumptions()
                .into_iter()
                .map(move |(sub, sup)| {
                    let sub: ClassExpression<ArcStr> = build.class(sub.id.clone()).into();
                    let sup: ClassExpression<ArcStr> = build.class(sup.id.clone()).into();
                    Component::SubClassOf(SubClassOf { sub, sup })
                }),
        )
    }

    fn is_consistent(&self) -> Result<bool, ReasonerError> {
        let build = ho::Build::<ArcStr>::new();
        self.is_entailed(&Component::SubClassOf(SubClassOf {
            sub: build.class(vocab::OWL::Thing.as_ref()).into(),
            sup: build.class(vocab::OWL::Nothing.as_ref()).into(),
        }))
        .map(|r| !r)
    }

    fn is_entailed(&self, cmp: &Component<ArcStr>) -> Result<bool, ReasonerError> {
        match cmp {
            Component::SubClassOf(SubClassOf {
                sub: ClassExpression::Class(sub),
                sup: ClassExpression::Class(sup),
            }) => Ok(self
                .state
                .named_subsumptions()
                .iter()
                .find(|(b, p)| sub.0.to_string() == b.id && sup.0.to_string() == p.id)
                .is_some()),
            c => Err(ReasonerError::Other(format!(
                "Cannot check entailment for component {:?}",
                c
            ))
            .into()),
        }
    }

    fn get_subclasses<'a>(
        &'a self,
        cmp: &'a ClassExpression<ArcStr>,
    ) -> Result<Box<dyn Iterator<Item = ClassExpression<ArcStr>> + 'a>, ReasonerError> {
        let build = ho::Build::<ArcStr>::new();

        match cmp {
            ClassExpression::Class(c) => Ok(Box::new(
                self.state
                    .named_subsumptions()
                    .into_iter()
                    .filter_map(move |(sub, sup)| {
                        if sup.id == c.to_string() {
                            Some(build.class(sub.id.clone()).into())
                        } else {
                            None
                        }
                    }),
            )),
            _ => Err(ReasonerError::Other(format!(
                "Cannot get subclasses for component {:?}",
                cmp
            ))
            .into()),
        }
    }
}
