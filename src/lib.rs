use horned_owl::model::{
    AnnotatedComponent, ArcStr, ClassExpression, Component, ForIRI,
    MutableOntology, SubClassOf,
};
use horned_owl::ontology::indexed::{OntologyIndex, TwoIndexedOntology};
use horned_owl::ontology::set::{SetIndex, SetOntology};
use horned_owl::{model as ho, vocab};
use std::collections::HashSet;
use std::sync::Arc;
use whelk::whelk::reasoner;

use pyhornedowlreasoner::{Reasoner, Reasoner2, ReasonerError};
use whelk::whelk::model::Axiom;
use whelk::whelk::owl::{translate_axiom, translate_ontology};
use whelk::whelk::reasoner::{assert, assert_append, ReasonerState};

#[derive(Default)]
pub struct PyWhelkIndex<A: ForIRI> {
    state: ReasonerState,
    pending_adds: HashSet<AnnotatedComponent<A>>,
    pending_deletes: HashSet<AnnotatedComponent<A>>,
}

pub struct PyWhelkReasoner<A: ForIRI>(
    TwoIndexedOntology<
        A,
        AnnotatedComponent<A>,
        SetIndex<A, AnnotatedComponent<A>>,
        PyWhelkIndex<A>,
    >,
);

#[unsafe(no_mangle)]
pub fn create_reasoner(ontology: SetOntology<ArcStr>) -> Box<dyn Reasoner2> {
    Box::new(PyWhelkReasoner::create_reasoner(ontology))
}

#[unsafe(no_mangle)]
pub fn create_incremental_reasoner(ontology: SetOntology<ArcStr>) -> Box<dyn Reasoner2> {
    Box::new(PyWhelkReasoner::create_reasoner(ontology))
}
impl PyWhelkReasoner<ArcStr> {
    fn create_reasoner(ontology: SetOntology<ArcStr>) -> Self {
        let translated = translate_ontology(&ontology);

        PyWhelkReasoner(TwoIndexedOntology::new(
            ontology.i().clone(),
            PyWhelkIndex {
                state: assert(&translated),
                ..Default::default()
            },
        ))
    }
}

impl OntologyIndex<ArcStr, AnnotatedComponent<ArcStr>> for PyWhelkIndex<ArcStr> {
    fn index_insert(&mut self, cmp: AnnotatedComponent<ArcStr>) -> bool {
        let translated = translate_axiom(&cmp.component)
            .into_iter()
            .filter_map(|c| match c.as_ref() {
                Axiom::ConceptInclusion(ci) => Some(ci.clone()),
                _ => None,
            })
            .collect();
        self.state = assert_append(&translated, &self.state);

        false
    }

    fn index_remove(&mut self, cmp: &AnnotatedComponent<ArcStr>) -> bool {
        false
    }
}

impl OntologyIndex<ArcStr, AnnotatedComponent<ArcStr>> for PyWhelkReasoner<ArcStr> {
    fn index_insert(&mut self, cmp: AnnotatedComponent<ArcStr>) -> bool {
        self.0.index_insert(cmp)
    }

    fn index_remove(&mut self, cmp: &AnnotatedComponent<ArcStr>) -> bool {
        self.0.index_remove(cmp)
    }

    fn index_take(&mut self, cmp: &AnnotatedComponent<ArcStr>) -> Option<AnnotatedComponent<ArcStr>> {
        self.0.index_take(cmp)
    }
}

impl Reasoner for PyWhelkReasoner<ArcStr> {
    fn classify(
        &self,
        ontology: &SetOntology<ArcStr>,
    ) -> Result<SetOntology<ArcStr>, ReasonerError> {
        let mut ontology: SetOntology<ArcStr> = ontology.clone();
        let inferred_components = self.infer(&ontology)?;

        for component in inferred_components {
            ontology.insert(ho::AnnotatedComponent {
                component,
                ann: Default::default(), // TODO: Add annotation stating this axiom is inferred
            });
        }

        Ok(ontology)
    }

    fn infer(
        &self,
        ontology: &SetOntology<ArcStr>,
    ) -> Result<HashSet<Component<ArcStr>>, ReasonerError> {
        let set_ontology: SetOntology<ArcStr> = ontology.clone();
        let build = ho::Build::<ArcStr>::new();

        let translated = translate_ontology(&set_ontology);
        let state = reasoner::assert(&translated);

        let inferred_components = state
            .named_subsumptions()
            .iter()
            .map(|(sub, sup)| {
                let sub: ho::ClassExpression<ArcStr> = build.class(sub.id.clone()).into();
                let sup: ho::ClassExpression<ArcStr> = build.class(sup.id.clone()).into();
                ho::Component::SubClassOf(ho::SubClassOf { sub, sup })
            })
            .collect();

        Ok(inferred_components)
    }
}

impl Reasoner2 for PyWhelkReasoner<ArcStr> {
    fn get_name(&self) -> String {
        "PyWhelk".to_string()
    }

    fn get_version(&self) -> String {
        "0.1.0".to_string()
    }

    fn get_ontology(&self) -> SetOntology<ArcStr> {
        SetOntology::from_index(self.0.i().clone())
    }

    fn is_consistent(&self) -> Result<bool, ReasonerError> {
        let build = ho::Build::<ArcStr>::new();
        self.is_entailed(&Component::SubClassOf(SubClassOf {
            sub: build.class(vocab::OWL::Thing.as_ref()).into(),
            sup: build.class(vocab::OWL::Nothing.as_ref()).into(),
        })).map(|r| !r)
    }

    fn is_entailed(&self, cmp: &Component<ArcStr>) -> Result<bool, ReasonerError> {
        match cmp {
            Component::SubClassOf(SubClassOf {
                sub: ClassExpression::Class(sub),
                sup: ClassExpression::Class(sup),
            }) => Ok(self
                .0
                .j()
                .state
                .named_subsumptions()
                .iter()
                .find(|(b, p)| {
                    sub.0.to_string() == b.id && sup.0.to_string() == p.id
                })
                .is_some()),
            _ => Err(ReasonerError::Other(
                "Cannot check entailment for this component".to_string(),
            )),
        }
    }
}
